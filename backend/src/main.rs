use axum::{
    Router,
    body::Bytes,
    extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::any,
};
use axum_extra::TypedHeader;
use tokio::sync::{OnceCell, mpsc};

use std::{net::Ipv4Addr, ops::ControlFlow, sync::Arc};
use std::{net::SocketAddr, path::PathBuf};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::CloseFrame;

//allows to split the websocket stream into separate TX and RX branches
use futures_util::{sink::SinkExt, stream::StreamExt};

// use log::{debug, error, info, warn};
use tracing::{debug, error, info, instrument::WithSubscriber, warn};

use clap::Parser;
use sqlx::types::time::OffsetDateTime;

mod db;
use db::DBHandle;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = Ipv4Addr::new(127, 0, 0, 1))]
    web_bind_addr: Ipv4Addr,
    #[arg(long, default_value_t = 4000)]
    web_port: u16,
}

static DB_HANDLE: OnceCell<Arc<DBHandle>> = OnceCell::const_new();
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./dist");

    // get the db handle
    let handle = DBHandle::open("data.db").await.unwrap();
    DB_HANDLE.set(handle).unwrap();

    // build our application with some routes
    let app = Router::new()
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/ws", any(ws_upgrade_handler))
        // logging
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    let listener =
        tokio::net::TcpListener::bind(SocketAddr::new(args.web_bind_addr.into(), args.web_port))
            .await
            .unwrap();
    debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

/// The handler for the HTTP request (this gets called when the HTTP request lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
async fn ws_upgrade_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    info!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_ws(socket, addr))
}

async fn handle_ws(socket: WebSocket, who: SocketAddr) {
    let (mut sender, mut receiver) = socket.split();
    let mut send_task = tokio::spawn(async move {
        let mut cnt = 0;
        loop {
            cnt += 1;
            match sender
                .send(Message::Text(format!("Server message {cnt} ...").into()))
                .await
            {
                Ok(_r) => {}
                Err(_e) => return cnt,
            }
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    });
    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = receiver.next().await {
            cnt += 1;
            // print message and break if instructed to do so
            if process_ws_message(msg, who).is_break() {
                break;
            }
        }
        cnt
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        rv_a = (&mut send_task) => {
            match rv_a {
                Ok(a) => debug!("{a} messages sent to {who}"),
                Err(a) => warn!("Error sending messages {a:?}")
            }
            recv_task.abort();
        },
        rv_b = (&mut recv_task) => {
            match rv_b {
                Ok(b) => debug!("Received {b} messages"),
                Err(b) => warn!("Error receiving messages {b:?}")
            }
            send_task.abort();
        }
    }
    info!("Websocket context {who} destroyed");
}

fn process_ws_message(msg: Message, who: SocketAddr) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            debug!(">>> {who} sent str: {t:?}");
        }
        Message::Binary(d) => {
            debug!(">>> {who} sent {} bytes: {d:?}", d.len());
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                info!(
                    ">>> {who} sent close with code {} and reason `{}`",
                    cf.code, cf.reason
                );
            } else {
                warn!(">>> {who} somehow sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            debug!(">>> {who} sent pong with {v:?}");
        }
        Message::Ping(v) => {
            debug!(">>> {who} sent ping with {v:?}");
        }
    }
    ControlFlow::Continue(())
}

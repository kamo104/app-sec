use axum::{
    Router,
    body::Body,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    routing::{any, get, post},
};
use axum_extra::TypedHeader;
use std::convert::Infallible;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::OnceCell;
use tower::Service;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

use std::net::{Ipv4Addr, SocketAddr};
use std::ops::ControlFlow;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;

//allows to split the websocket stream into separate TX and RX branches
use futures_util::{sink::SinkExt, stream::StreamExt};

// use log::{debug, error, info, warn};
use tracing::{debug, info, warn};

use clap::Parser;

mod db;
mod api;
mod generated;
mod email;
use db::DBHandle;
use api::{register_user, health_check, login_user, verify_email, get_counter, set_counter, logout_user, request_password_reset, complete_password_reset};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = Ipv4Addr::new(127, 0, 0, 1))]
    web_bind_addr: Ipv4Addr,
    #[arg(long, default_value_t = 4000)]
    web_port: u16,
    #[arg(long, default_value_t = false, help = "Run in development mode (uses data_dev.db with static key, no keyring required)")]
    dev: bool,
}

static DB_HANDLE: OnceCell<Arc<DBHandle>> = OnceCell::const_new();

/// A custom service that wraps ServeDir to provide SPA fallback behavior
/// For non-file requests, it serves index.html so Vue Router can handle the route
#[derive(Clone)]
struct SpaFallbackService {
    serve_dir: ServeDir,
    assets_dir: PathBuf,
}

impl SpaFallbackService {
    fn new(assets_dir: PathBuf) -> Self {
        Self {
            serve_dir: ServeDir::new(assets_dir.clone()).append_index_html_on_directories(true),
            assets_dir,
        }
    }
}

impl Service<Request<Body>> for SpaFallbackService {
    type Response = Response;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let uri = req.uri().clone();
        let path = uri.path();

        // Check if this is a file request (has extension)
        let is_file_request = path.contains('.') && !path.ends_with('/');

        // Clone the necessary data for the async block
        let mut serve_dir = self.serve_dir.clone();
        let assets_dir = self.assets_dir.clone();

        Box::pin(async move {
            // If it's a file request, try to serve it
            if is_file_request {
                // Use tower::Service::call directly
                match Service::<Request<Body>>::call(&mut serve_dir, req).await {
                    Ok(response) => Ok(response.map(Body::new)),
                    Err(_) => Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::from("File not found"))
                        .unwrap()),
                }
            } else {
                // For non-file requests (SPA routes), serve index.html
                let index_path = assets_dir.join("index.html");
                let contents = std::fs::read_to_string(&index_path)
                    .unwrap_or_else(|_| "Not Found".to_string());

                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html")
                    .body(Body::from(contents))
                    .unwrap())
            }
        })
    }
}

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
    let handle = if args.dev {
        info!("Running in development mode");
        DBHandle::open_dev().await.unwrap()
    } else {
        DBHandle::open("data.db").await.unwrap()
    };
    DB_HANDLE.set(handle).unwrap();

    // Duplicate DB handle for state
    let db_handle = DB_HANDLE.get().unwrap().clone();

    // build our application with some routes
    let app = Router::new()
        // API routes
        .route("/api/health", get(health_check))
        .route("/api/register", post(register_user))
        .route("/api/login", post(login_user))
        .route("/api/verify-email", post(verify_email))
        .route("/api/logout", post(logout_user))
        .route("/api/request-password-reset", post(request_password_reset))
        .route("/api/complete-password-reset", post(complete_password_reset))
        .route("/api/counter/get", get(get_counter))
        .route("/api/counter/set", post(set_counter))
        // WebSocket route
        .route("/ws", any(ws_upgrade_handler))
        // SPA fallback - serve index.html for all other routes
        .fallback_service(SpaFallbackService::new(assets_dir))
        // Add DB state
        .with_state(db_handle)
        // CORS layer - allow all origins in dev mode
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_headers(Any)
                .allow_origin(Any),
        )
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

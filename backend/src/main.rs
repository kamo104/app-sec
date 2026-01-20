use axum::{
    Router,
    body::Body,
    extract::{
        DefaultBodyLimit, FromRef,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{Request, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::any,
};
use axum_extra::TypedHeader;
use axum_server::tls_rustls::RustlsConfig;
use std::convert::Infallible;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::OnceCell;
use tower::Service;
use tower_cookies::CookieManagerLayer;
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use std::net::SocketAddr;
use std::ops::ControlFlow;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;

//allows to split the websocket stream into separate TX and RX branches
use futures_util::{sink::SinkExt, stream::StreamExt};

use tracing::{debug, error, info, warn};

mod api;
mod config;
mod db;
mod email;
use config::{Config, load_config};
use db::DBHandle;

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

#[derive(Clone)]
struct AppState {
    db: Arc<DBHandle>,
    config: Config,
}

#[tokio::main]
async fn main() {
    // Install rustls crypto provider before any TLS operations
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = load_config().expect("Failed to load config.toml");

    let assets_dir = std::env::var("ASSETS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./dist"));

    // get the db handle
    let handle = if config.server.dev_mode {
        info!("Running in development mode");
        DBHandle::open_dev(&config).await.unwrap()
    } else {
        DBHandle::open(&config.database.prod_path, &config)
            .await
            .unwrap()
    };
    DB_HANDLE.set(handle).unwrap();

    // Duplicate DB handle for state
    let db_handle = DB_HANDLE.get().unwrap().clone();

    // Start the cleanup task for expired sessions and tokens
    let _cleanup_handle = db_handle.clone().start_cleanup_task(config.clone());

    #[derive(Clone)]
    struct AppState {
        db: Arc<DBHandle>,
        config: Config,
    }

    impl FromRef<AppState> for Arc<DBHandle> {
        fn from_ref(input: &AppState) -> Self {
            input.db.clone()
        }
    }

    impl FromRef<AppState> for Config {
        fn from_ref(input: &AppState) -> Self {
            input.config.clone()
        }
    }

    let app_state = AppState {
        db: db_handle,
        config: config.clone(),
    };

    // Build OpenAPI router with all API routes
    let (api_router, openapi) = OpenApiRouter::with_openapi(
        utoipa::openapi::OpenApiBuilder::new()
            .info(
                utoipa::openapi::InfoBuilder::new()
                    .title("AppSec Demo API")
                    .version("1.0.0")
                    .description(Some(
                        "Web app demonstration focused on using good web security practices",
                    ))
                    .build(),
            )
            .build(),
    )
    .routes(routes!(api::health::health_check))
    .routes(routes!(api::register::register_user))
    .routes(routes!(api::login::login_user))
    .routes(routes!(api::auth::auth_check))
    .routes(routes!(api::auth::refresh_session))
    .routes(routes!(api::logout::logout_user))
    .routes(routes!(api::verify_email::verify_email))
    .routes(routes!(api::password_reset::request_password_reset))
    .routes(routes!(api::password_reset::complete_password_reset))
    .routes(routes!(api::counter::get_counter))
    .routes(routes!(api::counter::set_counter))
    // Posts endpoints
    .routes(routes!(api::posts::list_posts))
    .routes(routes!(api::posts::search_posts))
    .routes(routes!(api::posts::get_post))
    .routes(routes!(api::posts::get_post_image))
    .routes(routes!(api::posts::create_post))
    .routes(routes!(api::posts::update_post))
    .routes(routes!(api::posts::delete_post))
    // Comments endpoints
    .routes(routes!(api::comments::list_comments))
    .routes(routes!(api::comments::create_comment))
    .routes(routes!(api::comments::delete_comment))
    // Ratings endpoints
    .routes(routes!(api::ratings::rate_post))
    .routes(routes!(api::ratings::remove_rating))
    // Admin endpoints
    .routes(routes!(api::admin::list_users))
    .routes(routes!(api::admin::update_user_role))
    .routes(routes!(api::admin::delete_user))
    .routes(routes!(api::admin::list_deleted_posts))
    .routes(routes!(api::admin::restore_post))
    .split_for_parts();

    // Build the main application router
    let mut app = Router::new()
        .merge(api_router)
        .route("/ws", any(ws_upgrade_handler));

    // Add OpenAPI and Swagger UI in dev mode
    if config.server.dev_mode {
        app = app.merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", openapi.clone()));
        info!("OpenAPI spec available at /api/openapi.json");
        info!("Swagger UI available at /api/docs");
    }

    // Build CORS layer based on configuration
    let cors_layer = build_cors_layer(&config);

    // Build the app with all layers
    let mut app = app
        // SPA fallback - serve index.html for all other routes
        .fallback_service(SpaFallbackService::new(assets_dir))
        // Add app state
        .with_state(app_state)
        // Cookie management layer - must be before other layers
        .layer(CookieManagerLayer::new())
        // CORS layer
        .layer(cors_layer)
        .layer(DefaultBodyLimit::max(config.server.max_body_size))
        // logging
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // Add HSTS middleware if enabled (only makes sense with TLS)
    if config.security.hsts_enabled {
        let security_config = config.security.clone();
        info!(
            "HSTS enabled: max-age={}, includeSubDomains={}, preload={}",
            security_config.hsts_max_age_seconds,
            security_config.hsts_include_subdomains,
            security_config.hsts_preload
        );
        app = app.layer(middleware::from_fn(move |req, next| {
            hsts_middleware(req, next, security_config.clone())
        }));
    }

    let addr = SocketAddr::new(config.server.bind_addr.into(), config.server.port);

    // Start server with or without TLS
    if config.tls.enabled {
        info!("Starting HTTPS server on {}", addr);
        info!(
            "Using cert: {}, key: {}",
            config.tls.cert_path, config.tls.key_path
        );

        let tls_config =
            match RustlsConfig::from_pem_file(&config.tls.cert_path, &config.tls.key_path).await {
                Ok(cfg) => cfg,
                Err(e) => {
                    error!("Failed to load TLS certificates: {:?}", e);
                    error!(
                        "Cert path: {}, Key path: {}",
                        config.tls.cert_path, config.tls.key_path
                    );
                    panic!("TLS configuration failed");
                }
            };

        axum_server::bind_rustls(addr, tls_config)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    } else {
        if !config.server.dev_mode {
            warn!("TLS is disabled in production mode - this is insecure!");
        }
        debug!("Starting HTTP server on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        debug!("listening on {}", listener.local_addr().unwrap());
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    }
}

/// Build CORS layer based on configuration
fn build_cors_layer(config: &Config) -> CorsLayer {
    use tower_http::cors::AllowOrigin;

    // Common allowed methods
    let allowed_methods = [
        axum::http::Method::GET,
        axum::http::Method::POST,
        axum::http::Method::PUT,
        axum::http::Method::DELETE,
        axum::http::Method::PATCH,
        axum::http::Method::OPTIONS,
    ];

    // Common allowed headers
    let allowed_headers = [
        header::CONTENT_TYPE,
        header::AUTHORIZATION,
        header::ACCEPT,
        header::ORIGIN,
        header::COOKIE,
        header::HeaderName::from_static("x-requested-with"),
    ];

    if config.server.dev_mode {
        // In dev mode, allow localhost origins for easier testing
        // Note: allow_credentials(true) is incompatible with allow_origin(Any)
        info!("CORS: Permissive mode (dev) - allowing localhost origins");
        CorsLayer::new()
            .allow_methods(allowed_methods)
            .allow_headers(allowed_headers)
            .allow_origin(AllowOrigin::predicate(|origin, _req| {
                // Allow any localhost origin in dev mode
                origin.as_bytes().starts_with(b"http://localhost")
                    || origin.as_bytes().starts_with(b"http://127.0.0.1")
                    || origin.as_bytes().starts_with(b"https://localhost")
                    || origin.as_bytes().starts_with(b"https://127.0.0.1")
            }))
            .allow_credentials(true)
    } else if config.security.cors_allowed_origins.is_empty() {
        // In production with no configured origins, use restrictive defaults (same-origin)
        info!("CORS: Restrictive mode (same-origin only)");
        CorsLayer::new()
            .allow_methods(allowed_methods)
            .allow_headers(allowed_headers)
            .allow_credentials(true)
    } else {
        // Parse configured origins
        let origins: Vec<_> = config
            .security
            .cors_allowed_origins
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse().ok())
            .collect();

        info!("CORS: Configured origins: {:?}", origins);

        CorsLayer::new()
            .allow_methods(allowed_methods)
            .allow_headers(allowed_headers)
            .allow_origin(AllowOrigin::list(origins))
            .allow_credentials(true)
    }
}

/// HSTS middleware - adds Strict-Transport-Security header
async fn hsts_middleware(
    req: Request<Body>,
    next: Next,
    security_config: config::SecurityConfig,
) -> Response {
    let mut response = next.run(req).await;

    // Build HSTS header value
    let mut hsts_value = format!("max-age={}", security_config.hsts_max_age_seconds);
    if security_config.hsts_include_subdomains {
        hsts_value.push_str("; includeSubDomains");
    }
    if security_config.hsts_preload {
        hsts_value.push_str("; preload");
    }

    response.headers_mut().insert(
        header::STRICT_TRANSPORT_SECURITY,
        hsts_value.parse().unwrap(),
    );

    response
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

mod api;
mod auth;
mod db;
mod hub;
mod media;
mod ws;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{State, WebSocketUpgrade};
use axum::http::{header, uri::Authority, HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::Router;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::api::require_user;
use crate::db::Database;
use crate::hub::Hub;

pub struct AppState {
    pub db: Database,
    pub hub: Hub,
    pub upload_dir: PathBuf,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let bind: SocketAddr = std::env::var("LARPTRIX_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8080".into())
        .parse()
        .expect("LARPTRIX_BIND must be host:port");

    let db_path =
        PathBuf::from(std::env::var("LARPTRIX_DB").unwrap_or_else(|_| "data/larptrix.db".into()));
    let client_dir =
        PathBuf::from(std::env::var("LARPTRIX_CLIENT").unwrap_or_else(|_| "client".into()));
    let upload_dir =
        PathBuf::from(std::env::var("LARPTRIX_UPLOADS").unwrap_or_else(|_| "data/uploads".into()));
    std::fs::create_dir_all(&upload_dir).expect("upload dir");

    let db = Database::open(&db_path).expect("open sqlite");
    db.ping().expect("sqlite ping");

    let state = Arc::new(AppState {
        db,
        hub: Hub::new(),
        upload_dir,
    });

    let app = Router::new()
        .route("/", get(|| async { Redirect::to("/index.html") }))
        .route("/ws", get(ws_upgrade))
        .route("/health", get(health))
        .merge(api::router())
        .fallback_service(ServeDir::new(client_dir))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    tracing::info!("listening on http://{bind}");
    let listener = tokio::net::TcpListener::bind(bind).await.expect("bind");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("serve");
}

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if !same_origin(&headers) {
        return (StatusCode::FORBIDDEN, "cross-origin WebSocket request").into_response();
    }
    let server_name = match api::matrix_server_name(&headers) {
        Ok(name) => name,
        Err(err) => return err.into_response(),
    };

    match require_user(&state, &headers) {
        Ok(user) => {
            ws.on_upgrade(move |socket| ws::handle_socket(socket, state, user, server_name))
        }
        Err(err) => err.into_response(),
    }
}

fn same_origin(headers: &HeaderMap) -> bool {
    let mut origins = headers.get_all(header::ORIGIN).iter();
    let Some(origin) = origins.next().and_then(|value| value.to_str().ok()) else {
        return false;
    };
    if origins.next().is_some() {
        return false;
    }

    let Ok(origin) = origin.parse::<Uri>() else {
        return false;
    };
    let Some(scheme) = origin.scheme_str() else {
        return false;
    };
    if !matches!(scheme, "http" | "https") {
        return false;
    }

    let expected_scheme = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.split(',').next().unwrap_or_default().trim())
        .unwrap_or("http");
    if !scheme.eq_ignore_ascii_case(expected_scheme) {
        return false;
    }

    let Some(origin_authority) = origin.authority() else {
        return false;
    };
    if origin_authority.as_str().contains('@') {
        return false;
    }
    let Some(host) = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<Authority>().ok())
    else {
        return false;
    };

    origin_authority.host().eq_ignore_ascii_case(host.host())
        && effective_port(origin_authority, scheme) == effective_port(&host, scheme)
}

fn effective_port(authority: &Authority, scheme: &str) -> Option<u16> {
    authority.port_u16().or(match scheme {
        "http" => Some(80),
        "https" => Some(443),
        _ => None,
    })
}

async fn health() -> &'static str {
    "ok\n"
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutting down");
}

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::same_origin;
    use axum::http::{header, HeaderMap, HeaderValue};

    fn request_headers(origin: &str, host: &str, scheme: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::ORIGIN, HeaderValue::from_str(origin).unwrap());
        headers.insert(header::HOST, HeaderValue::from_str(host).unwrap());
        headers.insert("x-forwarded-proto", HeaderValue::from_str(scheme).unwrap());
        headers
    }

    #[test]
    fn websocket_origin_must_match_scheme_and_authority() {
        assert!(same_origin(&request_headers(
            "https://chat.example.test",
            "chat.example.test",
            "https"
        )));
        assert!(same_origin(&request_headers(
            "http://localhost:8080",
            "localhost:8080",
            "http"
        )));
        assert!(!same_origin(&request_headers(
            "https://attacker.example.test",
            "chat.example.test",
            "https"
        )));
        assert!(!same_origin(&request_headers(
            "https://chat.example.test.attacker.example",
            "chat.example.test",
            "https"
        )));
        assert!(!same_origin(&request_headers(
            "http://chat.example.test",
            "chat.example.test",
            "https"
        )));
    }

    #[test]
    fn websocket_origin_is_required_and_must_be_unambiguous() {
        assert!(!same_origin(&HeaderMap::new()));

        let mut headers =
            request_headers("https://chat.example.test", "chat.example.test", "https");
        headers.append(
            header::ORIGIN,
            HeaderValue::from_static("https://attacker.example.test"),
        );
        assert!(!same_origin(&headers));
    }
}

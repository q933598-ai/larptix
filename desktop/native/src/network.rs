use std::sync::Arc;
use std::time::Duration;

use async_channel::{Receiver, Sender};
use futures_util::{SinkExt, StreamExt};
use larptrix_protocol::{ClientMessage, ServerMessage, UserInfo};
use reqwest::cookie::{CookieStore, Jar};
use serde::Serialize;
use tokio::time::{interval, sleep};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use url::Url;

pub enum AppEvent {
    Authenticated {
        user: UserInfo,
        outgoing: Sender<ClientMessage>,
    },
    Server(ServerMessage),
    Status(String),
    Error(String),
}

#[derive(Serialize)]
struct LoginRequest<'a> {
    access_key: &'a str,
    device_name: &'static str,
}

pub async fn connect(
    server_address: String,
    access_key: String,
    events: Sender<AppEvent>,
) -> Result<(), String> {
    let base = normalize_server_url(&server_address)?;
    let jar = Arc::new(Jar::default());
    let http = reqwest::Client::builder()
        .cookie_provider(Arc::clone(&jar))
        .user_agent(concat!("Larptrix-Native/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| format!("Could not create HTTP client: {error}"))?;

    let login_url = base
        .join("api/login")
        .map_err(|error| format!("Invalid server URL: {error}"))?;

    events
        .send(AppEvent::Status("Signing in…".into()))
        .await
        .map_err(|_| "Application closed".to_string())?;

    let response = http
        .post(login_url)
        .json(&LoginRequest {
            access_key: &access_key,
            device_name: "Larptrix Native",
        })
        .send()
        .await
        .map_err(|error| format!("Could not reach the server: {error}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let detail = response.text().await.unwrap_or_default();
        let detail = detail.trim();
        return Err(if detail.is_empty() {
            format!("Sign-in failed ({status})")
        } else {
            format!("Sign-in failed ({status}): {detail}")
        });
    }

    let user: UserInfo = response
        .json()
        .await
        .map_err(|error| format!("Server returned an invalid login response: {error}"))?;

    let cookie = jar
        .cookies(&base)
        .ok_or_else(|| "The server did not issue a session cookie".to_string())?
        .to_str()
        .map_err(|_| "The server issued an invalid session cookie".to_string())?
        .to_string();

    let origin = base.origin().ascii_serialization();
    let (outgoing, commands) = async_channel::bounded::<ClientMessage>(128);

    events
        .send(AppEvent::Authenticated {
            user,
            outgoing: outgoing.clone(),
        })
        .await
        .map_err(|_| "Application closed".to_string())?;

    let mut retry_delay = Duration::from_secs(1);

    loop {
        events
            .send(AppEvent::Status("Connecting to live updates…".into()))
            .await
            .map_err(|_| "Application closed".to_string())?;

        let mut ws_url = base
            .join("ws")
            .map_err(|error| format!("Invalid WebSocket URL: {error}"))?;
        let ws_scheme = match ws_url.scheme() {
            "https" => "wss",
            "http" => "ws",
            _ => return Err("Only HTTP and HTTPS server URLs are supported".into()),
        };
        ws_url
            .set_scheme(ws_scheme)
            .map_err(|_| "Could not create WebSocket URL".to_string())?;

        let mut request = ws_url
            .as_str()
            .into_client_request()
            .map_err(|error| format!("Could not create WebSocket request: {error}"))?;

        request.headers_mut().insert(
            tokio_tungstenite::tungstenite::http::header::ORIGIN,
            origin
                .parse()
                .map_err(|_| "Could not set WebSocket Origin header".to_string())?,
        );
        request.headers_mut().insert(
            tokio_tungstenite::tungstenite::http::header::COOKIE,
            cookie
                .parse()
                .map_err(|_| "Could not set session cookie on WebSocket".to_string())?,
        );

        match tokio_tungstenite::connect_async(request).await {
            Ok((socket, _response)) => {
                retry_delay = Duration::from_secs(1);
                events
                    .send(AppEvent::Status("Connected · waiting for server state…".into()))
                    .await
                    .map_err(|_| "Application closed".to_string())?;

                run_socket(socket, &commands, &events).await;
                events
                    .send(AppEvent::Status("Connection lost · reconnecting…".into()))
                    .await
                    .map_err(|_| "Application closed".to_string())?;
            }
            Err(error) => {
                let _ = events
                    .send(AppEvent::Status(format!(
                        "Live connection failed: {error}"
                    )))
                    .await;
            }
        }

        sleep(retry_delay).await;
        retry_delay = (retry_delay * 2).min(Duration::from_secs(30));
    }
}

fn normalize_server_url(raw: &str) -> Result<Url, String> {
    let mut url = Url::parse(raw.trim())
        .map_err(|error| format!("Invalid server URL: {error}"))?;

    if !matches!(url.scheme(), "http" | "https") {
        return Err("Server URL must start with http:// or https://".into());
    }
    if url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() {
        return Err("Server URL must contain a valid host and no embedded credentials".into());
    }

    url.set_query(None);
    url.set_fragment(None);

    if !url.path().ends_with('/') {
        let path = format!("{}/", url.path());
        url.set_path(&path);
    }

    Ok(url)
}

async fn run_socket(
    socket: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    commands: &Receiver<ClientMessage>,
    events: &Sender<AppEvent>,
) {
    let (mut writer, mut reader) = socket.split();
    let mut heartbeat = interval(Duration::from_secs(25));
    heartbeat.tick().await;

    loop {
        tokio::select! {
            frame = reader.next() => {
                match frame {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ServerMessage>(text.as_str()) {
                            Ok(message) => {
                                if events.send(AppEvent::Server(message)).await.is_err() {
                                    break;
                                }
                            }
                            Err(error) => {
                                let _ = events.send(AppEvent::Status(format!(
                                    "Ignored an unknown server event: {error}"
                                ))).await;
                            }
                        }
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        if writer.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    Some(Ok(Message::Pong(_))) | Some(Ok(Message::Binary(_))) => {}
                    Some(Ok(Message::Frame(_))) => {}
                }
            }
            command = commands.recv() => {
                let Ok(command) = command else {
                    break;
                };
                let Ok(text) = serde_json::to_string(&command) else {
                    continue;
                };
                if writer.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
            _ = heartbeat.tick() => {
                let ping = ClientMessage::Ping;
                let Ok(text) = serde_json::to_string(&ping) else {
                    continue;
                };
                if writer.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
        }
    }
}

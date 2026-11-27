use std::{path::PathBuf, sync::Arc};
use std::time::Duration;

use async_channel::{Receiver, Sender};
use futures_util::{SinkExt, StreamExt};
use larptrix_protocol::{ChatMessage, ClientMessage, ServerMessage, UserInfo};
use serde_json::{json, Value};
use reqwest::cookie::{CookieStore, Jar};
use serde::{Deserialize, Serialize};
use tokio::time::{interval, sleep};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use url::Url;

use crate::crypto::{EncryptedAttachmentMetadata, MatrixToDeviceEvent, NativeMatrixCrypto};
use crate::media::MediaPlayback;

pub enum AppEvent {
    Authenticated {
        user: UserInfo,
        outgoing: Sender<ClientMessage>,
        crypto: Arc<NativeMatrixCrypto>,
        api: Arc<NativeApi>,
    },
    FriendListsLoaded {
        friends: Vec<UserInfo>,
        incoming: Vec<UserInfo>,
        outgoing: Vec<UserInfo>,
    },
    UserSearchResults {
        users: Vec<UserInfo>,
    },
    Server(ServerMessage),
    Decrypted {
        peer_id: String,
        message_id: String,
        body: String,
        attachments: Vec<EncryptedAttachmentMetadata>,
    },
    FilesPicked(Vec<PickedFile>),
    ImageCached {
        attachment_id: String,
        path: PathBuf,
        error: Option<String>,
    },
    MediaOpened {
        session_id: String,
        title: String,
        is_video: bool,
        path: PathBuf,
        player: Arc<MediaPlayback>,
    },
    MediaTick {
        session_id: String,
        position_ms: u64,
        duration_ms: u64,
        finished: bool,
        error: Option<String>,
    },
    MediaFailed {
        session_id: String,
        error: String,
    },
    MessageSendFinished {
        body: String,
        success: bool,
        error: Option<String>,
    },
    DecryptionFailed {
        peer_id: String,
        message: ChatMessage,
        error: String,
    },
    E2ESynced,
    Status(String),
    Error(String),
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FriendLists {
    #[serde(default)]
    pub friends: Vec<UserInfo>,
    #[serde(default)]
    pub incoming: Vec<UserInfo>,
    #[serde(default)]
    pub outgoing: Vec<UserInfo>,
}

#[derive(Clone)]
pub struct NativeApi {
    http: reqwest::Client,
    base_url: Url,
}

impl NativeApi {
    fn endpoint(&self, path: &str) -> Result<Url, String> {
        self.base_url
            .join(path.trim_start_matches('/'))
            .map_err(|error| format!("Invalid API endpoint: {error}"))
    }

    async fn read_json(response: reqwest::Response) -> Result<Value, String> {
        let status = response.status();
        let body = response.text().await
            .map_err(|error| format!("Could not read server response: {error}"))?;
        if !status.is_success() {
            return Err(if body.trim().is_empty() {
                format!("Server request failed ({status})")
            } else {
                format!("Server request failed ({status}): {}", body.trim())
            });
        }
        serde_json::from_str(&body)
            .map_err(|error| format!("Server returned invalid JSON: {error}"))
    }

    async fn get_json(&self, path: &str) -> Result<Value, String> {
        let url = self.endpoint(path)?;
        let response = self.http.get(url).send().await
            .map_err(|error| format!("Could not reach server API: {error}"))?;
        Self::read_json(response).await
    }

    async fn post_json(&self, path: &str, body: Value) -> Result<Value, String> {
        let url = self.endpoint(path)?;
        let response = self.http.post(url).json(&body).send().await
            .map_err(|error| format!("Could not reach server API: {error}"))?;
        Self::read_json(response).await
    }

    pub async fn load_friends(&self) -> Result<FriendLists, String> {
        let value = self.get_json("api/friends").await?;
        serde_json::from_value(value)
            .map_err(|error| format!("Could not decode friend list: {error}"))
    }

    pub async fn search_users(&self, query: &str) -> Result<Vec<UserInfo>, String> {
        let mut url = self.endpoint("api/users/search")?;
        url.query_pairs_mut().append_pair("q", query);
        let response = self.http.get(url).send().await
            .map_err(|error| format!("Could not search users: {error}"))?;
        let value = Self::read_json(response).await?;
        serde_json::from_value(value.get("users").cloned().unwrap_or_else(|| json!([])))
            .map_err(|error| format!("Could not decode user search results: {error}"))
    }

    pub async fn send_friend_request(&self, user_id: &str) -> Result<(), String> {
        let path = format!("api/friends/{user_id}");
        self.post_json(&path, json!({})).await?;
        Ok(())
    }

    pub async fn accept_friend_request(&self, user_id: &str) -> Result<(), String> {
        let path = format!("api/friends/{user_id}/accept");
        self.post_json(&path, json!({})).await?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct PickedFile {
    pub path: PathBuf,
    pub name: String,
    pub size_bytes: u64,
}

#[derive(Clone, Debug)]
pub enum LoginCredentials {
    AccessKey(String),
    EmailPassword { email: String, password: String },
}

#[derive(Serialize)]
struct LoginRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    access_key: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<&'a str>,
    device_name: &'static str,
}

pub async fn connect(
    server_address: String,
    credentials: LoginCredentials,
    recovery_key: String,
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
        .send_blocking(AppEvent::Status("Signing in…".into()))
        .map_err(|_| "Application closed".to_string())?;

    let response = http
        .post(login_url)
        .json(&match &credentials {
            LoginCredentials::AccessKey(access_key) => LoginRequest {
                access_key: Some(access_key),
                email: None,
                password: None,
                device_name: "Larptrix Native",
            },
            LoginCredentials::EmailPassword { email, password } => LoginRequest {
                access_key: None,
                email: Some(email),
                password: Some(password),
                device_name: "Larptrix Native",
            },
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

    events
        .send_blocking(AppEvent::Status("Unlocking Matrix E2E device…".into()))
        .map_err(|_| "Application closed".to_string())?;

    let crypto = NativeMatrixCrypto::initialize(
        http.clone(),
        base.clone(),
        user.user_id.clone(),
        recovery_key,
    )
    .await?;

    let api = Arc::new(NativeApi {
        http: http.clone(),
        base_url: base.clone(),
    });

    let cookie = jar
        .cookies(&base)
        .ok_or_else(|| "The server did not issue a session cookie".to_string())?
        .to_str()
        .map_err(|_| "The server issued an invalid session cookie".to_string())?
        .to_string();

    let origin = base.origin().ascii_serialization();
    let (outgoing, commands) = async_channel::bounded::<ClientMessage>(128);

    events
        .send_blocking(AppEvent::Authenticated {
            user,
            outgoing: outgoing.clone(),
            crypto: crypto.clone(),
            api: api.clone(),
        })
        .map_err(|_| "Application closed".to_string())?;

    {
        let api = api.clone();
        let events = events.clone();
        tokio::spawn(async move {
            match api.load_friends().await {
                Ok(data) => {
                    let _ = events.send_blocking(AppEvent::FriendListsLoaded {
                        friends: data.friends,
                        incoming: data.incoming,
                        outgoing: data.outgoing,
                    });
                }
                Err(error) => {
                    let _ = events.send_blocking(AppEvent::Status(format!(
                        "Could not load friends: {error}"
                    )));
                }
            }
        });
    }

    let mut retry_delay = Duration::from_secs(1);

    loop {
        events
            .send_blocking(AppEvent::Status("Connecting to live updates…".into()))
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
                if let Err(error) = crypto.process_pending_to_device().await {
                    let _ = events.send_blocking(AppEvent::Status(format!(
                        "E2E sync will retry: {error}"
                    )));
                }
                events
                    .send_blocking(AppEvent::Status("Connected · waiting for server state…".into()))
                    .map_err(|_| "Application closed".to_string())?;

                run_socket(socket, commands.clone(), events.clone(), crypto.clone()).await;
                events
                    .send_blocking(AppEvent::Status("Connection lost · reconnecting…".into()))
                    .map_err(|_| "Application closed".to_string())?;
            }
            Err(error) => {
                let _ = events
                    .send_blocking(AppEvent::Status(format!(
                        "Live connection failed: {error}"
                    )));
            }
        }

        sleep(retry_delay).await;
        retry_delay = (retry_delay * 2).min(Duration::from_secs(30));
    }
}

pub(crate) fn normalize_server_url(raw: &str) -> Result<Url, String> {
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
    commands: Receiver<ClientMessage>,
    events: Sender<AppEvent>,
    crypto: Arc<NativeMatrixCrypto>,
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
                            Ok(ServerMessage::MatrixToDevice {
                                event_id,
                                sender_id,
                                sender_device_id: _,
                                recipient_device_id,
                                event_type,
                                txn_id,
                                content,
                            }) => {
                                let event = MatrixToDeviceEvent {
                                    event_id,
                                    sender_id,
                                    recipient_device_id,
                                    event_type,
                                    txn_id,
                                    content,
                                };
                                match crypto.handle_to_device_event(&event).await {
                                    Ok(()) => {
                                        let _ = events.send_blocking(AppEvent::E2ESynced);
                                    }
                                    Err(error) => {
                                        let _ = events.send_blocking(AppEvent::Status(format!(
                                            "E2E device sync pending: {error}"
                                        )));
                                    }
                                }
                            }
                            Ok(message) => {
                                if events.send_blocking(AppEvent::Server(message)).is_err() {
                                    break;
                                }
                            }
                            Err(error) => {
                                let _ = events.send_blocking(AppEvent::Status(format!(
                                    "Ignored an unknown server event: {error}"
                                )));
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


#[cfg(test)]
mod tests {
    use super::normalize_server_url;

    #[test]
    fn normalizes_server_addresses_for_api_and_websocket_paths() {
        let url = normalize_server_url("https://chat.example.test").unwrap();
        assert_eq!(url.as_str(), "https://chat.example.test/");
        assert_eq!(url.join("api/login").unwrap().as_str(), "https://chat.example.test/api/login");
        assert_eq!(url.join("ws").unwrap().as_str(), "https://chat.example.test/ws");
    }

    #[test]
    fn preserves_a_reverse_proxy_path_prefix() {
        let url = normalize_server_url("https://example.test/larptrix").unwrap();
        assert_eq!(
            url.join("api/login").unwrap().as_str(),
            "https://example.test/larptrix/api/login"
        );
    }

    #[test]
    fn rejects_insecure_schemes_and_embedded_credentials() {
        assert!(normalize_server_url("ftp://example.test").is_err());
        assert!(normalize_server_url("https://user:password@example.test").is_err());
    }

    #[test]
    fn removes_query_and_fragment_from_server_addresses() {
        let url = normalize_server_url("https://example.test/chat?token=secret#fragment").unwrap();
        assert_eq!(url.as_str(), "https://example.test/chat/");
    }
}

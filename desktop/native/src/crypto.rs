use std::{
    collections::BTreeMap,
    env,
    path::{Path, PathBuf},
    sync::Arc,
};

use base64::{engine::general_purpose::{STANDARD, STANDARD_NO_PAD}, Engine as _};
use matrix_sdk_crypto::{
    decrypt_room_key_export, encrypt_room_key_export,
    DecryptionSettings, EncryptionSettings, EncryptionSyncChanges, OlmMachine,
    OlmMachineBuilder, TrustRequirement,
    types::{events::room::encrypted::EncryptedEvent, requests::AnyOutgoingRequest},
};
use matrix_sdk_sqlite::SqliteCryptoStore;
use http::{Response as HttpResponse, StatusCode};
use reqwest::{Client, Method};
use ruma::{
    api::{
        IncomingResponseExt, OutgoingRequestExt,
        auth_scheme::SendAccessToken,
        client::{
        keys::{
            claim_keys::v3::Response as KeysClaimResponse,
            get_keys::v3::Response as KeysQueryResponse,
            upload_keys::v3::Response as KeysUploadResponse,
        },
        sync::sync_events::DeviceLists,
        to_device::send_event_to_device::v3::Response as ToDeviceResponse,
        },
    },
    events::{AnyMessageLikeEventContent, AnyToDeviceEvent},
    serde::Raw,
    OwnedDeviceId, OwnedUserId, RoomId, UserId,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct MatrixToDeviceEvent {
    pub event_id: i64,
    pub sender_id: String,
    pub recipient_device_id: String,
    pub event_type: String,
    pub txn_id: String,
    pub content: Value,
}

pub struct NativeMatrixCrypto {
    http: Client,
    base_url: Url,
    server_name: String,
    internal_user_id: String,
    matrix_user_id: OwnedUserId,
    device_id: OwnedDeviceId,
    recovery_key: String,
    machine: OlmMachine,
    decryption_settings: DecryptionSettings,
    outgoing_lock: Mutex<()>,
    room_lock: Mutex<()>,
}

impl NativeMatrixCrypto {
    pub async fn initialize(
        http: Client,
        base_url: Url,
        internal_user_id: String,
        recovery_key: String,
    ) -> Result<Arc<Self>, String> {
        validate_recovery_key(&recovery_key)?;

        let config = api_request(&http, &base_url, Method::GET, "api/matrix/config", None).await?;
        let server_name = config
            .get("server_name")
            .and_then(Value::as_str)
            .filter(|name| !name.trim().is_empty())
            .ok_or_else(|| "The server did not return its Matrix server name".to_string())?
            .to_string();
        let matrix_user_id = UserId::parse(format!("@{}:{}", internal_user_id, server_name))
            .map_err(|error| format!("Invalid Matrix account id: {error}"))?;

        let account_dir = crypto_data_dir(&internal_user_id)?;
        std::fs::create_dir_all(&account_dir)
            .map_err(|error| format!("Could not create encrypted key directory: {error}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&account_dir, std::fs::Permissions::from_mode(0o700))
                .map_err(|error| format!("Could not secure encrypted key directory: {error}"))?;
        }
        let device_id = load_or_create_device_id(&account_dir)?;
        let store_dir = account_dir.join("matrix-store");
        let store = SqliteCryptoStore::open(&store_dir, Some(&recovery_key))
            .await
            .map_err(|error| format!("Could not open encrypted Matrix key store: {error}"))?;
        let machine = OlmMachineBuilder::new(&matrix_user_id, &device_id)
            .with_crypto_store(store)
            .build()
            .await
            .map_err(|error| format!("Could not initialize Matrix E2E: {error}"))?;

        // Match the web client: recover missing Megolm keys from other devices
        // and only automatically forward keys to verified devices.
        machine.set_room_key_requests_enabled(true);
        machine.set_room_key_forwarding_enabled(true);

        let client = Arc::new(Self {
            http,
            base_url,
            server_name,
            internal_user_id,
            matrix_user_id,
            device_id,
            recovery_key: recovery_key.clone(),
            machine,
            decryption_settings: DecryptionSettings {
                sender_device_trust_requirement: TrustRequirement::Untrusted,
            },
            outgoing_lock: Mutex::new(()),
            room_lock: Mutex::new(()),
        });

        client.restore_room_key_backup().await?;

        // Match the web client's bootstrap so a fresh device publishes its
        // identity and one-time keys before attempting to send a message.
        let changed_devices = DeviceLists::default();
        let key_counts = BTreeMap::new();
        client.machine.receive_sync_changes(
            EncryptionSyncChanges {
                to_device_events: Vec::new(),
                changed_devices: &changed_devices,
                one_time_keys_counts: &key_counts,
                unused_fallback_keys: None,
                next_batch_token: None,
            },
            &client.decryption_settings,
        ).await.map_err(|error| format!("Could not initialize device keys: {error}"))?;

        client.process_outgoing_requests().await?;
        let _ = client.process_pending_to_device().await;
        let _ = client.sync_room_key_backup().await;
        Ok(client)
    }

    pub fn device_id(&self) -> &str {
        self.device_id.as_str()
    }

    pub fn room_id_for_dm(&self, peer_id: &str) -> String {
        let mut ids = [self.internal_user_id.as_str(), peer_id];
        ids.sort_unstable();
        let pair = ids.join("\n");
        let digest = Sha256::digest(format!("{}\n{}", self.server_name, pair).as_bytes());
        format!("!larpdm_{:x}:{}", digest, self.server_name)
    }

    pub fn room_id_for_group(&self, group_id: &str) -> String {
        format!("!larpgrp_{}:{}", group_id, self.server_name)
    }

    pub async fn encrypt_dm_text(&self, peer_id: &str, text: &str) -> Result<String, String> {
        let room_id = self.room_id_for_dm(peer_id);
        self.encrypt_room_payload(&room_id, &[peer_id.to_string()], json!({ "text": text })).await
    }

    pub async fn decrypt_message(
        &self,
        message_id: &str,
        sender_id: &str,
        created_at: i64,
        envelope: &Value,
    ) -> Result<Value, String> {
        if envelope.get("version").and_then(Value::as_i64) != Some(3)
            || envelope.get("message_type").and_then(Value::as_str) != Some("matrix")
        {
            return Err("Unsupported encrypted message format".into());
        }

        let room_id = envelope.get("room_id").and_then(Value::as_str)
            .ok_or_else(|| "Encrypted message is missing its room id".to_string())?;
        let ciphertext = envelope.get("ciphertext")
            .cloned()
            .ok_or_else(|| "Encrypted message is missing ciphertext".to_string())?;
        let event_id = ["$", message_id].concat();
        let raw_event_value = json!({
            "event_id": event_id,
            "sender": format!("@{}:{}", sender_id, self.server_name),
            "origin_server_ts": created_at,
            "room_id": room_id,
            "type": "m.room.encrypted",
            "content": ciphertext,
        });
        let raw_event = Raw::new(&raw_event_value)
            .map_err(|error| format!("Invalid encrypted event: {error}"))?
            .cast_unchecked::<EncryptedEvent>();
        let room_id = RoomId::parse(room_id.to_string())
            .map_err(|error| format!("Invalid Matrix room id: {error}"))?;
        let decrypted = match self.machine.decrypt_room_event(
            &raw_event, &room_id, &self.decryption_settings,
        ).await {
            Ok(decrypted) => decrypted,
            Err(error) => {
                // A missing Megolm key may have queued an m.room_key_request.
                // Send that request so another device can share the room key.
                let message = format!("Could not decrypt Matrix message: {error}");
                let _ = self.process_outgoing_requests().await;
                return Err(message);
            }
        };
        let event: Value = serde_json::from_str(decrypted.event.json().get())
            .map_err(|error| format!("Invalid decrypted Matrix event: {error}"))?;
        Ok(event.get("content").cloned().unwrap_or(event))
    }

    async fn encrypt_room_payload(
        &self,
        room_id: &str,
        peer_ids: &[String],
        payload: Value,
    ) -> Result<String, String> {
        let _room_guard = self.room_lock.lock().await;
        self.prepare_room(room_id, peer_ids).await?;
        let room_id = RoomId::parse(room_id.to_string())
            .map_err(|error| format!("Invalid Matrix room id: {error}"))?;
        let content = Raw::new(&payload)
            .map_err(|error| format!("Could not encode message: {error}"))?
            .cast_unchecked::<AnyMessageLikeEventContent>();
        let encrypted = self.machine.encrypt_room_event_raw(&room_id, "m.room.message", &content)
            .await.map_err(|error| format!("Could not encrypt message: {error}"))?;
        let ciphertext = serde_json::to_value(encrypted.content)
            .map_err(|error| format!("Could not serialize ciphertext: {error}"))?;
        Ok(json!({
            "version": 3,
            "message_type": "matrix",
            "sender_device_id": self.device_id.as_str(),
            "room_id": room_id.as_str(),
            "ciphertext": ciphertext
        }).to_string())
    }

    async fn prepare_room(&self, room_id: &str, peer_ids: &[String]) -> Result<(), String> {
        let room_id = RoomId::parse(room_id.to_string())
            .map_err(|error| format!("Invalid Matrix room id: {error}"))?;
        let mut peers = peer_ids.iter()
            .filter(|id| !id.is_empty() && id.as_str() != self.internal_user_id)
            .cloned().collect::<Vec<_>>();
        peers.sort();
        peers.dedup();

        let mut users = Vec::<OwnedUserId>::with_capacity(peers.len() + 1);
        users.push(self.matrix_user_id.clone());
        for peer in peers {
            users.push(UserId::parse(format!("@{}:{}", peer, self.server_name))
                .map_err(|error| format!("Invalid peer Matrix id: {error}"))?);
        }
        self.machine.update_tracked_users(users.iter().skip(1).map(|user| user.as_ref()))
            .await.map_err(|error| format!("Could not track Matrix devices: {error}"))?;

        if users.len() > 1 {
            let (request_id, request) = self.machine.query_keys_for_users(
                users.iter().skip(1).map(|user| user.as_ref()),
            );
            let (status, response_body) = self.api_post_raw("api/matrix/keys/query", json!({
                "device_keys": request.device_keys,
                "timeout": request.timeout.map(|value| value.as_millis() as u64)
            })).await?;
            let response: KeysQueryResponse = parse_matrix_response(status, &response_body)?;
            self.machine.mark_request_as_sent(&request_id, &response).await
                .map_err(|error| format!("Could not apply device keys: {error}"))?;
        }

        self.process_outgoing_requests().await?;
        if let Some((request_id, request)) = self.machine.get_missing_sessions(
            users.iter().map(|user| user.as_ref()),
        ).await.map_err(|error| format!("Could not find missing Olm sessions: {error}"))? {
            let encoded_request = (*request).clone()
                .try_into_http_request::<Vec<u8>>(
                    "https://matrix.invalid",
                    SendAccessToken::Always("serialization-only"),
                    (),
                )
                .map_err(|error| format!("Could not encode Matrix key claim: {error}"))?;
            let request_body: Value = serde_json::from_slice(encoded_request.body())
                .map_err(|error| format!("Could not decode encoded Matrix key claim: {error}"))?;
            let (status, response_body) = self.api_post_raw("api/matrix/keys/claim", request_body).await?;
            let response: KeysClaimResponse = parse_matrix_response(status, &response_body)?;
            self.machine.mark_request_as_sent(&request_id, &response).await
                .map_err(|error| format!("Could not establish Olm sessions: {error}"))?;
        }

        let requests = self.machine.share_room_key(
            &room_id,
            users.iter().map(|user| user.as_ref()),
            EncryptionSettings::default(),
        ).await.map_err(|error| format!("Could not share Megolm room key: {error}"))?;
        for request in requests {
            let body = json!({
                "device_id": self.device_id.as_str(),
                "event_type": request.event_type.to_string(),
                "txn_id": request.txn_id.to_string(),
                "messages": serde_json::to_value(&request.messages)
                    .map_err(|error| format!("Could not encode to-device messages: {error}"))?
            });
            let (status, response_body) = self.api_post_raw("api/matrix/to-device", body).await?;
            let response: ToDeviceResponse = parse_matrix_response(status, &response_body)?;
            self.machine.mark_request_as_sent(&request.txn_id, &response).await
                .map_err(|error| format!("Could not commit room-key delivery: {error}"))?;
        }
        self.process_outgoing_requests().await?;
        let _ = self.sync_room_key_backup().await;
        Ok(())
    }

    async fn restore_room_key_backup(&self) -> Result<(), String> {
        let response = self.api_get("api/matrix/key-backup").await?;
        let Some(encrypted) = response.get("encrypted_backup").and_then(Value::as_str) else {
            return Ok(());
        };
        if encrypted.is_empty() {
            return Ok(());
        }
        let exported = decrypt_room_key_export(encrypted.as_bytes(), &self.recovery_key)
            .map_err(|error| format!("Could not unlock the existing E2E room-key backup: {error}"))?;
        self.machine.store().import_room_keys(exported, None, |_, _| {})
            .await.map_err(|error| format!("Could not restore E2E room keys: {error}"))?;
        Ok(())
    }

    async fn sync_room_key_backup(&self) -> Result<(), String> {
        let exported = self.machine.store().export_room_keys(|_| true).await
            .map_err(|error| format!("Could not export E2E room keys: {error}"))?;
        if exported.is_empty() {
            return Ok(());
        }
        let encrypted = encrypt_room_key_export(&exported, &self.recovery_key, 100_000)
            .map_err(|error| format!("Could not encrypt E2E room-key backup: {error}"))?;
        self.api_request_put_key_backup(encrypted).await
    }

    async fn api_request_put_key_backup(&self, encrypted_backup: String) -> Result<(), String> {
        api_request(
            &self.http,
            &self.base_url,
            Method::PUT,
            "api/matrix/key-backup",
            Some(json!({ "encrypted_backup": encrypted_backup })),
        ).await.map(|_| ())
    }

    pub async fn process_pending_to_device(&self) -> Result<(), String> {
        let response = self.api_get("api/matrix/to-device/pending").await?;
        let events = response.get("events").and_then(Value::as_array)
            .cloned().unwrap_or_default();
        for event in events {
            self.process_to_device_event(&event).await?;
        }
        Ok(())
    }

    pub async fn handle_to_device_event(&self, event: &MatrixToDeviceEvent) -> Result<(), String> {
        self.process_to_device_event(&json!({
            "id": event.event_id,
            "sender_id": event.sender_id,
            "recipient_device_id": event.recipient_device_id,
            "event_type": event.event_type,
            "txn_id": event.txn_id,
            "content": event.content
        })).await
    }

    async fn process_to_device_event(&self, value: &Value) -> Result<(), String> {
        let event_id = value.get("id").and_then(Value::as_i64)
            .ok_or_else(|| "Matrix to-device event has no id".to_string())?;
        let sender = value.get("sender_id").and_then(Value::as_str)
            .ok_or_else(|| "Matrix to-device event has no sender".to_string())?;
        let recipient_device_id = value.get("recipient_device_id").and_then(Value::as_str)
            .ok_or_else(|| "Matrix to-device event has no recipient".to_string())?;
        if recipient_device_id != self.device_id.as_str() {
            return Err("Matrix to-device event targets another device".into());
        }
        let event_type = value.get("event_type").and_then(Value::as_str)
            .ok_or_else(|| "Matrix to-device event has no type".to_string())?;
        let txn_id = value.get("txn_id").and_then(Value::as_str).unwrap_or_default();
        let content = value.get("content").cloned().unwrap_or_else(|| json!({}));
        let sender = if sender.starts_with('@') {
            sender.to_string()
        } else {
            format!("@{}:{}", sender, self.server_name)
        };
        let raw_value = json!({
            "type": event_type,
            "sender": sender,
            "content": content,
            "unsigned": { "transaction_id": txn_id }
        });
        let raw_event = Raw::new(&raw_value)
            .map_err(|error| format!("Invalid Matrix to-device event: {error}"))?
            .cast_unchecked::<AnyToDeviceEvent>();
        let changed_devices = DeviceLists::default();
        let key_counts = BTreeMap::new();
        let unused_fallback_keys: [ruma::OneTimeKeyAlgorithm; 0] = [];
        self.machine.receive_sync_changes_msc4186(
            EncryptionSyncChanges {
                to_device_events: vec![raw_event],
                changed_devices: &changed_devices,
                one_time_keys_counts: &key_counts,
                unused_fallback_keys: Some(&unused_fallback_keys),
                next_batch_token: None,
            },
            &self.decryption_settings,
        ).await.map_err(|error| format!("Could not process Matrix to-device event: {error}"))?;
        self.process_outgoing_requests().await?;
        self.api_post("api/matrix/to-device/ack", json!({ "event_ids": [event_id] })).await?;
        let _ = self.sync_room_key_backup().await;
        Ok(())
    }

    async fn process_outgoing_requests(&self) -> Result<(), String> {
        let _guard = self.outgoing_lock.lock().await;
        loop {
            let requests = self.machine.outgoing_requests().await
                .map_err(|error| format!("Could not read Matrix crypto requests: {error}"))?;
            if requests.is_empty() {
                break;
            }
            for outgoing in requests {
            let request_id = outgoing.request_id().to_owned();
            match outgoing.request() {
                AnyOutgoingRequest::KeysUpload(request) => {
                    let encoded_request = (*request).clone()
                        .try_into_http_request::<Vec<u8>>(
                            "https://matrix.invalid",
                            SendAccessToken::Always("serialization-only"),
                            (),
                        )
                        .map_err(|error| format!("Could not encode Matrix key upload: {error}"))?;
                    let request_body: Value = serde_json::from_slice(encoded_request.body())
                        .map_err(|error| format!("Could not decode encoded Matrix key upload: {error}"))?;
                    let (status, response_body) = self.api_post_raw("api/matrix/keys/upload", json!({
                        "device_id": self.device_id.as_str(),
                        "request": request_body
                    })).await?;
                    let response: KeysUploadResponse = parse_matrix_response(status, &response_body)?;
                    self.machine.mark_request_as_sent(&request_id, &response).await
                        .map_err(|error| format!("Could not apply key-upload response: {error}"))?;
                }
                AnyOutgoingRequest::KeysQuery(request) => {
                    let (status, response_body) = self.api_post_raw("api/matrix/keys/query", json!({
                        "device_keys": request.device_keys,
                        "timeout": request.timeout.map(|value| value.as_millis() as u64)
                    })).await?;
                    let response: KeysQueryResponse = parse_matrix_response(status, &response_body)?;
                    self.machine.mark_request_as_sent(&request_id, &response).await
                        .map_err(|error| format!("Could not apply key-query response: {error}"))?;
                }
                AnyOutgoingRequest::KeysClaim(request) => {
                    let encoded_request = (*request).clone()
                        .try_into_http_request::<Vec<u8>>(
                            "https://matrix.invalid",
                            SendAccessToken::Always("serialization-only"),
                            (),
                        )
                        .map_err(|error| format!("Could not encode Matrix key claim: {error}"))?;
                    let request_body: Value = serde_json::from_slice(encoded_request.body())
                        .map_err(|error| format!("Could not decode encoded Matrix key claim: {error}"))?;
                    let (status, response_body) = self.api_post_raw("api/matrix/keys/claim", request_body).await?;
                    let response: KeysClaimResponse = parse_matrix_response(status, &response_body)?;
                    self.machine.mark_request_as_sent(&request_id, &response).await
                        .map_err(|error| format!("Could not apply key-claim response: {error}"))?;
                }
                AnyOutgoingRequest::ToDeviceRequest(request) => {
                    let body = json!({
                        "device_id": self.device_id.as_str(),
                        "event_type": request.event_type.to_string(),
                        "txn_id": request.txn_id.to_string(),
                        "messages": serde_json::to_value(&request.messages)
                            .map_err(|error| format!("Could not encode to-device messages: {error}"))?
                    });
                    let (status, response_body) = self.api_post_raw("api/matrix/to-device", body).await?;
                    let response: ToDeviceResponse = parse_matrix_response(status, &response_body)?;
                    self.machine.mark_request_as_sent(&request_id, &response).await
                        .map_err(|error| format!("Could not apply to-device response: {error}"))?;
                }
                AnyOutgoingRequest::SignatureUpload(_) | AnyOutgoingRequest::RoomMessage(_) => {
                    return Err("Native interactive device verification is not implemented yet".into());
                }
            }
        }
        }
        Ok(())
    }

    async fn api_get(&self, path: &str) -> Result<Value, String> {
        api_request(&self.http, &self.base_url, Method::GET, path, None).await
    }

    async fn api_post(&self, path: &str, body: Value) -> Result<Value, String> {
        api_request(&self.http, &self.base_url, Method::POST, path, Some(body)).await
    }

    async fn api_post_raw(
        &self,
        path: &str,
        body: Value,
    ) -> Result<(StatusCode, Vec<u8>), String> {
        let url = self.base_url.join(path)
            .map_err(|error| format!("Invalid Larptrix API path: {error}"))?;
        let response = self.http.post(url).json(&body).send().await
            .map_err(|error| format!("Could not reach Larptrix E2E API: {error}"))?;
        let status = response.status();
        let bytes = response.bytes().await
            .map_err(|error| format!("Could not read Larptrix E2E response: {error}"))?;
        if !status.is_success() {
            return Err(format!(
                "Larptrix E2E API returned {}: {}",
                status,
                String::from_utf8_lossy(&bytes).trim()
            ));
        }
        Ok((status, bytes.to_vec()))
    }
}

fn parse_matrix_response<T>(status: StatusCode, body: &[u8]) -> Result<T, String>
where
    T: IncomingResponseExt,
{
    let response = HttpResponse::builder()
        .status(status)
        .body(body)
        .map_err(|error| format!("Could not build Matrix API response: {error}"))?;
    T::try_from_http_response(response)
        .map_err(|error| format!("Could not parse Matrix API response: {error}"))
}

fn validate_recovery_key(key: &str) -> Result<(), String> {
    let decoded = STANDARD_NO_PAD.decode(key).or_else(|_| STANDARD.decode(key))
        .map_err(|_| "E2E recovery key must be base64".to_string())?;
    if decoded.len() != 32 {
        return Err("E2E recovery key must decode to exactly 32 bytes".into());
    }
    Ok(())
}

fn crypto_data_dir(user_id: &str) -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    let root = env::var_os("APPDATA").map(PathBuf::from)
        .ok_or_else(|| "Windows APPDATA is unavailable".to_string())?;
    #[cfg(not(target_os = "windows"))]
    let root = match env::var_os("XDG_DATA_HOME") {
        Some(path) => PathBuf::from(path),
        None => env::var_os("HOME").map(PathBuf::from)
            .ok_or_else(|| "HOME is unavailable".to_string())?
            .join(".local").join("share"),
    };
    let hash = Sha256::digest(user_id.as_bytes());
    Ok(root.join("Larptrix").join("crypto").join(format!("{:x}", hash)))
}

fn load_or_create_device_id(directory: &Path) -> Result<OwnedDeviceId, String> {
    let path = directory.join("device-id");
    if path.is_file() {
        let value = std::fs::read_to_string(&path)
            .map_err(|error| format!("Could not read Matrix device id: {error}"))?;
        let value = value.trim();
        if !value.is_empty() && value.len() <= 255 {
            return Ok(value.to_string().into());
        }
    }
    let value = Uuid::new_v4().simple().to_string();
    std::fs::write(&path, &value)
        .map_err(|error| format!("Could not save Matrix device id: {error}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .map_err(|error| format!("Could not secure Matrix device id: {error}"))?;
    }
    Ok(value.into())
}

async fn api_request(
    http: &Client,
    base_url: &Url,
    method: Method,
    path: &str,
    body: Option<Value>,
) -> Result<Value, String> {
    let url = base_url.join(path)
        .map_err(|error| format!("Invalid Larptrix API path: {error}"))?;
    let mut request = http.request(method, url);
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request.send().await
        .map_err(|error| format!("Could not reach Larptrix E2E API: {error}"))?;
    let status = response.status();
    let bytes = response.bytes().await
        .map_err(|error| format!("Could not read Larptrix E2E response: {error}"))?;
    if !status.is_success() {
        return Err(format!("Larptrix E2E API returned {}: {}", status, String::from_utf8_lossy(&bytes).trim()));
    }
    if bytes.is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("Larptrix E2E API returned invalid JSON: {error}"))
}

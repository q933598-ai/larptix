use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    env,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
use base64::{engine::general_purpose::{STANDARD, STANDARD_NO_PAD}, Engine as _};
use matrix_sdk_crypto::{
    decrypt_room_key_export, encrypt_room_key_export,
    DecryptionSettings, EncryptionSettings, EncryptionSyncChanges, OlmMachine,
    OlmMachineBuilder, TrustRequirement,
    types::{events::room::encrypted::EncryptedEvent, requests::AnyOutgoingRequest},
};
use matrix_sdk_sqlite::SqliteCryptoStore;
use flate2::read::GzDecoder;
use http::{Response as HttpResponse, StatusCode};
use reqwest::{multipart, Client, Method};
use ruma::{
    api::{
        IncomingResponseExt, OutgoingRequestExt, SupportedVersions,
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
use serde::{Deserialize, Serialize};
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedAttachmentMetadata {
    pub id: String,
    pub url: String,
    pub key: String,
    pub iv: String,
    pub name: String,
    pub mime: String,
    pub size: u64,
    pub stored_size: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compression: Option<String>,
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
    prepared_rooms: Mutex<HashMap<String, Instant>>,
    backup_lock: Mutex<()>,
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
            prepared_rooms: Mutex::new(HashMap::new()),
            backup_lock: Mutex::new(()),
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
        matrix_dm_room_id(&self.server_name, &self.internal_user_id, peer_id)
    }

    pub fn room_id_for_group(&self, group_id: &str) -> String {
        matrix_group_room_id(group_id, &self.server_name)
    }

    pub async fn encrypt_dm_text(&self, peer_id: &str, text: &str) -> Result<String, String> {
        self.encrypt_dm_payload(peer_id, json!({ "text": text })).await
    }

    pub async fn encrypt_dm_payload(&self, peer_id: &str, payload: Value) -> Result<String, String> {
        let room_id = self.room_id_for_dm(peer_id);
        self.encrypt_room_payload(&room_id, &[peer_id.to_string()], payload).await
    }

    pub async fn encrypt_group_text(
        &self,
        group_id: &str,
        member_ids: &[String],
        text: &str,
    ) -> Result<String, String> {
        self.encrypt_group_payload(group_id, member_ids, json!({ "text": text })).await
    }

    pub async fn encrypt_group_payload(
        &self,
        group_id: &str,
        member_ids: &[String],
        payload: Value,
    ) -> Result<String, String> {
        let room_id = self.room_id_for_group(group_id);
        self.encrypt_room_payload(&room_id, member_ids, payload).await
    }

    pub async fn encrypt_attachment_and_upload(
        &self,
        name: &str,
        mime: &str,
        plaintext: &[u8],
    ) -> Result<EncryptedAttachmentMetadata, String> {
        if plaintext.is_empty() {
            return Err("Cannot attach an empty file".into());
        }
        if plaintext.len().saturating_add(16) > larptrix_protocol::MAX_ATTACHMENT_BYTES {
            return Err("Encrypted attachment exceeds the 25 MiB server limit".into());
        }

        let mut key = [0u8; 32];
        let mut iv = [0u8; 12];
        getrandom::getrandom(&mut key)
            .map_err(|error| format!("Could not generate attachment key: {error}"))?;
        getrandom::getrandom(&mut iv)
            .map_err(|error| format!("Could not generate attachment nonce: {error}"))?;

        let ciphertext = encrypt_attachment_bytes(plaintext, &key, &iv)?;
        // Keep the original filename only inside the Matrix-encrypted payload.
        let upload_name = "attachment.encrypted";
        let upload_url = self.base_url.join("api/upload")
            .map_err(|error| format!("Invalid upload URL: {error}"))?;
        let part = multipart::Part::bytes(ciphertext.clone())
            .file_name(upload_name)
            .mime_str("application/octet-stream")
            .map_err(|error| format!("Could not prepare encrypted attachment: {error}"))?;
        let form = multipart::Form::new().part("file", part);
        let response = self.http.post(upload_url)
            .multipart(form)
            .send()
            .await
            .map_err(|error| format!("Could not upload encrypted attachment: {error}"))?;
        let status = response.status();
        let response_body = response.bytes().await
            .map_err(|error| format!("Could not read upload response: {error}"))?;
        if !status.is_success() {
            let detail = String::from_utf8_lossy(&response_body);
            return Err(format!("Attachment upload failed ({status}): {}", detail.trim()));
        }
        let uploaded: Value = serde_json::from_slice(&response_body)
            .map_err(|error| format!("Server returned an invalid attachment response: {error}"))?;
        let id = uploaded.get("id").and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "Server did not return an attachment id".to_string())?
            .to_string();
        let url = uploaded.get("url").and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| larptrix_protocol::attachment_url(&id));

        Ok(EncryptedAttachmentMetadata {
            id,
            url,
            key: STANDARD.encode(key),
            iv: STANDARD.encode(iv),
            name: name.to_string(),
            mime: if mime.trim().is_empty() {
                "application/octet-stream".into()
            } else {
                mime.to_string()
            },
            size: plaintext.len() as u64,
            stored_size: ciphertext.len() as u64,
            compression: None,
        })
    }

    pub async fn download_and_decrypt_attachment(
        &self,
        metadata: &EncryptedAttachmentMetadata,
    ) -> Result<Vec<u8>, String> {
        // Do not trust the URL carried inside the peer-authored encrypted payload.
        // Build the download route from our configured server and the opaque attachment ID.
        if metadata.id.is_empty() || metadata.id.len() > 128 {
            return Err("Invalid attachment id".into());
        }
        let mut url = self.base_url.join("api/attachments/")
            .map_err(|error| format!("Invalid attachment API URL: {error}"))?;
        url.path_segments_mut()
            .map_err(|_| "Attachment API URL cannot accept a path segment".to_string())?
            .push(&metadata.id);

        let mut response = self.http.get(url).send().await
            .map_err(|error| format!("Could not download encrypted attachment: {error}"))?;
        let status = response.status();
        if !status.is_success() {
            return Err(format!("Attachment download failed ({status})"));
        }
        if response.content_length().is_some_and(|size| size > larptrix_protocol::MAX_ATTACHMENT_BYTES as u64) {
            return Err("Downloaded attachment exceeds the server size limit".into());
        }
        let mut ciphertext = Vec::new();
        while let Some(chunk) = response.chunk().await
            .map_err(|error| format!("Could not read encrypted attachment: {error}"))?
        {
            if ciphertext.len().saturating_add(chunk.len()) > larptrix_protocol::MAX_ATTACHMENT_BYTES {
                return Err("Downloaded attachment exceeds the server size limit".into());
            }
            ciphertext.extend_from_slice(&chunk);
        }
        if ciphertext.len() as u64 != metadata.stored_size {
            return Err("Downloaded attachment size does not match its metadata".into());
        }

        let key = STANDARD.decode(&metadata.key)
            .or_else(|_| STANDARD_NO_PAD.decode(&metadata.key))
            .map_err(|_| "Attachment key is not valid base64".to_string())?;
        let iv = STANDARD.decode(&metadata.iv)
            .or_else(|_| STANDARD_NO_PAD.decode(&metadata.iv))
            .map_err(|_| "Attachment nonce is not valid base64".to_string())?;
        let key: [u8; 32] = key.try_into()
            .map_err(|_| "Attachment key must contain exactly 32 bytes".to_string())?;
        let iv: [u8; 12] = iv.try_into()
            .map_err(|_| "Attachment nonce must contain exactly 12 bytes".to_string())?;

        let mut plaintext = decrypt_attachment_bytes(&ciphertext, &key, &iv)?;
        match metadata.compression.as_deref() {
            Some("gzip") => {
                let decoder = GzDecoder::new(plaintext.as_slice());
                let mut decompressed = Vec::new();
                decoder
                    .take(larptrix_protocol::MAX_ATTACHMENT_BYTES as u64 + 1)
                    .read_to_end(&mut decompressed)
                    .map_err(|error| format!("Could not decompress attachment: {error}"))?;
                if decompressed.len() > larptrix_protocol::MAX_ATTACHMENT_BYTES {
                    return Err("Decompressed attachment exceeds the 25 MiB limit".into());
                }
                plaintext = decompressed;
            }
            Some(other) => return Err(format!("Unsupported attachment compression: {other}")),
            None => {}
        }
        if plaintext.len() as u64 != metadata.size {
            return Err("Decrypted attachment size does not match its metadata".into());
        }
        Ok(plaintext)
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
        let raw_event_value = matrix_encrypted_event(
            message_id,
            sender_id,
            created_at,
            room_id,
            &self.server_name,
            ciphertext,
        );
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
        Ok(matrix_message_envelope(
            self.device_id.as_str(),
            room_id.as_str(),
            ciphertext,
        ).to_string())
    }

    async fn prepare_room(&self, room_id: &str, peer_ids: &[String]) -> Result<(), String> {
        let room_id = RoomId::parse(room_id.to_string())
            .map_err(|error| format!("Invalid Matrix room id: {error}"))?;
        let mut peers = peer_ids.iter()
            .filter(|id| !id.is_empty() && id.as_str() != self.internal_user_id)
            .cloned().collect::<Vec<_>>();
        peers.sort();
        peers.dedup();

        let cache_key = format!("{}|{}", room_id, peers.join(","));
        {
            let prepared_rooms = self.prepared_rooms.lock().await;
            if prepared_rooms
                .get(&cache_key)
                .is_some_and(|prepared_at| prepared_at.elapsed() < Duration::from_secs(30))
            {
                return Ok(());
            }
        }

        let mut users = Vec::<OwnedUserId>::with_capacity(peers.len() + 1);
        users.push(self.matrix_user_id.clone());
        for peer in peers {
            users.push(UserId::parse(format!("@{}:{}", peer, self.server_name))
                .map_err(|error| format!("Invalid peer Matrix id: {error}"))?);
        }
        // Materialize each SDK input as an owned Vec. This avoids passing
        // opaque iterator adaptors through SDK futures spawned by the UI.
        let user_refs: Vec<&UserId> = users.iter().map(|user| user.as_ref()).collect();
        let tracked_user_refs: Vec<&UserId> = user_refs.iter().skip(1).copied().collect();
        self.machine
            .update_tracked_users(tracked_user_refs)
            .await
            .map_err(|error| format!("Could not track Matrix devices: {error}"))?;

        if user_refs.len() > 1 {
            let key_query_users: Vec<&UserId> = user_refs.iter().skip(1).copied().collect();
            let (request_id, request) = self.machine.query_keys_for_users(key_query_users);
            let (status, response_body) = self.api_post_raw("api/matrix/keys/query", json!({
                "device_keys": request.device_keys,
                "timeout": request.timeout.map(|value| value.as_millis() as u64)
            })).await?;
            let response: KeysQueryResponse = parse_matrix_response(status, &response_body)?;
            self.machine.mark_request_as_sent(&request_id, &response).await
                .map_err(|error| format!("Could not apply device keys: {error}"))?;
        }

        self.process_outgoing_requests().await?;
        let missing_session_users: Vec<&UserId> = user_refs.to_vec();
        if let Some((request_id, request)) = self.machine.get_missing_sessions(
            missing_session_users.into_iter(),
        ).await.map_err(|error| format!("Could not find missing Olm sessions: {error}"))? {
            let encoded_request = request
                .try_into_http_request::<Vec<u8>>(
                    "https://matrix.invalid",
                    SendAccessToken::Always("serialization-only"),
                    Cow::Owned(matrix_supported_versions()),
                )
                .map_err(|error| format!("Could not encode Matrix key claim: {error}"))?;
            let request_body: Value = serde_json::from_slice(encoded_request.body())
                .map_err(|error| format!("Could not decode encoded Matrix key claim: {error}"))?;
            let (status, response_body) = self.api_post_raw("api/matrix/keys/claim", request_body).await?;
            let response: KeysClaimResponse = parse_matrix_response(status, &response_body)?;
            self.machine.mark_request_as_sent(&request_id, &response).await
                .map_err(|error| format!("Could not establish Olm sessions: {error}"))?;
        }

        let room_key_share_users: Vec<&UserId> = user_refs.to_vec();
        let requests = self.machine.share_room_key(
            &room_id,
            room_key_share_users.into_iter(),
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
        self.prepared_rooms.lock().await.insert(cache_key, Instant::now());
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
        // Serialize snapshots so an older export cannot overwrite a newer backup.
        let _guard = self.backup_lock.lock().await;
        let exported = self.machine.store().export_room_keys(|_| true).await
            .map_err(|error| format!("Could not export E2E room keys: {error}"))?;
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
        let mut first_error = None;
        for event in events {
            if let Err(error) = self.process_to_device_event(&event).await {
                first_error.get_or_insert(error);
            }
        }
        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
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
                            Cow::Owned(matrix_supported_versions()),
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
                            Cow::Owned(matrix_supported_versions()),
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

fn encrypt_attachment_bytes(plaintext: &[u8], key: &[u8; 32], iv: &[u8; 12]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| "Invalid attachment encryption key".to_string())?;
    cipher.encrypt(Nonce::from_slice(iv), plaintext)
        .map_err(|_| "Could not encrypt attachment bytes".to_string())
}

fn decrypt_attachment_bytes(ciphertext: &[u8], key: &[u8; 32], iv: &[u8; 12]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| "Invalid attachment decryption key".to_string())?;
    cipher.decrypt(Nonce::from_slice(iv), ciphertext)
        .map_err(|_| "Attachment authentication failed: wrong key, nonce, or damaged file".to_string())
}

fn matrix_group_room_id(group_id: &str, server_name: &str) -> String {
    format!("!larpgrp_{group_id}:{server_name}")
}

fn matrix_dm_room_id(server_name: &str, first_user_id: &str, second_user_id: &str) -> String {
    let mut ids = [first_user_id, second_user_id];
    ids.sort_unstable();
    let pair = ids.join("\n");
    let digest = Sha256::digest(format!("{}\n{}", server_name, pair).as_bytes());
    format!("!larpdm_{:x}:{}", digest, server_name)
}

fn matrix_message_envelope(sender_device_id: &str, room_id: &str, ciphertext: Value) -> Value {
    json!({
        "version": 3,
        "message_type": "matrix",
        "sender_device_id": sender_device_id,
        "room_id": room_id,
        "ciphertext": ciphertext
    })
}

fn matrix_encrypted_event(
    message_id: &str,
    sender_id: &str,
    created_at: i64,
    room_id: &str,
    server_name: &str,
    ciphertext: Value,
) -> Value {
    json!({
        "event_id": format!("${}", message_id),
        "sender": format!("@{}:{}", sender_id, server_name),
        "origin_server_ts": created_at,
        "room_id": room_id,
        "type": "m.room.encrypted",
        "content": ciphertext
    })
}
fn matrix_supported_versions() -> SupportedVersions {
    // The Larptrix Matrix API implements the stable v1.0 endpoint paths used
    // for device-key upload and one-time-key claiming.
    let versions = vec!["v1.0".to_string()];
    SupportedVersions::from_parts(&versions, &BTreeMap::new())
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

#[cfg(test)]
mod interoperability_tests {
    use super::*;
    use serde_json::Value;

    fn fixture() -> Value {
        serde_json::from_str(include_str!("../../../tests/fixtures/matrix-interop.json"))
            .expect("shared Matrix interoperability fixture must be valid JSON")
    }

    #[test]
    fn dm_room_id_matches_web_client_golden_vector() {
        let fixture = fixture();
        let server_name = fixture["server_name"].as_str().unwrap();
        let local_user_id = fixture["local_user_id"].as_str().unwrap();
        let peer_user_id = fixture["peer_user_id"].as_str().unwrap();
        let expected = fixture["room_id"].as_str().unwrap();

        assert_eq!(matrix_dm_room_id(server_name, local_user_id, peer_user_id), expected);
        assert_eq!(matrix_dm_room_id(server_name, peer_user_id, local_user_id), expected);
    }

    #[test]
    fn aes_gcm_attachment_roundtrip_matches_web_crypto_layout() {
        let key = [7u8; 32];
        let iv = [9u8; 12];
        let plaintext = b"Larptrix encrypted attachment";
        let ciphertext = encrypt_attachment_bytes(plaintext, &key, &iv).unwrap();

        assert_eq!(ciphertext.len(), plaintext.len() + 16);
        assert_eq!(decrypt_attachment_bytes(&ciphertext, &key, &iv).unwrap(), plaintext);
    }

    #[test]
    fn aes_gcm_attachment_matches_shared_browser_fixture() {
        let fixture = fixture()["attachment_crypto"].clone();
        let key: [u8; 32] = STANDARD
            .decode(fixture["key"].as_str().unwrap())
            .unwrap()
            .try_into()
            .unwrap();
        let iv: [u8; 12] = STANDARD
            .decode(fixture["iv"].as_str().unwrap())
            .unwrap()
            .try_into()
            .unwrap();
        let plaintext = fixture["plaintext"].as_str().unwrap().as_bytes();
        let expected_ciphertext = STANDARD
            .decode(fixture["ciphertext"].as_str().unwrap())
            .unwrap();

        let encrypted = encrypt_attachment_bytes(plaintext, &key, &iv).unwrap();
        assert_eq!(encrypted, expected_ciphertext);
        assert_eq!(encrypted.len() as u64, fixture["stored_size"].as_u64().unwrap());
        assert_eq!(plaintext.len() as u64, fixture["size"].as_u64().unwrap());
        assert_eq!(decrypt_attachment_bytes(&expected_ciphertext, &key, &iv).unwrap(), plaintext);
    }

    #[test]
    fn aes_gcm_attachment_rejects_tampered_ciphertext() {
        let key = [7u8; 32];
        let iv = [9u8; 12];
        let mut ciphertext = encrypt_attachment_bytes(b"protected bytes", &key, &iv).unwrap();
        ciphertext[0] ^= 0x80;

        assert!(decrypt_attachment_bytes(&ciphertext, &key, &iv).is_err());
    }

    #[test]
    fn group_room_id_matches_the_web_client_contract() {
        let fixture = fixture();
        assert_eq!(
            matrix_group_room_id(
                fixture["group_id"].as_str().unwrap(),
                fixture["server_name"].as_str().unwrap(),
            ),
            fixture["group_room_id"].as_str().unwrap(),
        );
    }

    #[test]
    fn browser_and_native_encode_identical_matrix_message_content() {
        let fixture = fixture();
        let browser_payload_json = fixture["payload_json"].as_str().unwrap();
        let browser_content: Raw<AnyMessageLikeEventContent> =
            serde_json::from_str(browser_payload_json).unwrap();
        let native_content = Raw::new(&fixture["payload"])
            .unwrap()
            .cast_unchecked::<AnyMessageLikeEventContent>();

        // JSON object member order is not semantically significant. The web
        // serializes insertion order, while serde_json::Value may sort keys.
        let browser_value: Value = serde_json::from_str(browser_content.json().get()).unwrap();
        let native_value: Value = serde_json::from_str(native_content.json().get()).unwrap();
        assert_eq!(browser_value, fixture["payload"]);
        assert_eq!(native_value, fixture["payload"]);
        assert_eq!(serde_json::from_str::<Value>(browser_payload_json).unwrap(), native_value);
    }

    #[test]
    fn native_message_envelope_matches_web_client_contract() {
        let fixture = fixture();
        let envelope = matrix_message_envelope(
            fixture["sender_device_id"].as_str().unwrap(),
            fixture["room_id"].as_str().unwrap(),
            fixture["ciphertext"].clone(),
        );
        assert_eq!(envelope, fixture["envelope"]);
        assert_eq!(envelope["version"], 3);
        assert_eq!(envelope["message_type"], "matrix");
    }

    #[test]
    fn encrypted_event_reconstruction_matches_web_client_contract() {
        let fixture = fixture();
        let event = matrix_encrypted_event(
            fixture["message_id"].as_str().unwrap(),
            fixture["sender_id"].as_str().unwrap(),
            fixture["created_at"].as_i64().unwrap(),
            fixture["room_id"].as_str().unwrap(),
            fixture["server_name"].as_str().unwrap(),
            fixture["ciphertext"].clone(),
        );
        assert_eq!(event, fixture["encrypted_event"]);
    }

    #[test]
    fn web_recovery_key_shape_is_accepted_by_native_client() {
        let fixture = fixture();
        validate_recovery_key(fixture["recovery_key"].as_str().unwrap()).unwrap();
    }

    #[test]
    fn matrix_room_key_backup_roundtrips_with_web_parameters() {
        let fixture = fixture();
        let recovery_key = fixture["recovery_key"].as_str().unwrap();
        let exported_keys = Vec::<matrix_sdk_crypto::olm::ExportedRoomKey>::new();

        let encrypted = encrypt_room_key_export(&exported_keys, recovery_key, 100_000).unwrap();
        let restored = decrypt_room_key_export(encrypted.as_bytes(), recovery_key).unwrap();

        assert!(restored.is_empty());
    }
}

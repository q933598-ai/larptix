use async_channel::Sender;
use std::{collections::{HashMap, HashSet}, path::PathBuf, sync::Arc};
use gpui::{
    div, img, prelude::*, rgb, px, App, Context, FocusHandle, KeyDownEvent, MouseButton, Window,
};
use larptrix_client_core::ClientCore;
use larptrix_protocol::{ChatMessage, ClientMessage, ServerMessage};
use tokio::runtime::Handle;

use crate::crypto::{EncryptedAttachmentMetadata, NativeMatrixCrypto};
use crate::network::{self, AppEvent, PickedFile};

const BG: u32 = 0x1b1f2a;
const RAIL: u32 = 0x171b24;
const SIDEBAR: u32 = 0x222a38;
const SURFACE: u32 = 0x2a3344;
const SURFACE_HOVER: u32 = 0x333f54;
const BORDER: u32 = 0x3a4559;
const TEXT: u32 = 0xd8dee9;
const MUTED: u32 = 0x9aa8bd;
const ACCENT: u32 = 0x88c0d0;
const GREEN: u32 = 0xa3be8c;
const RED: u32 = 0xbf616a;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Section {
    #[default]
    Messages,
    Friends,
    Groups,
    Settings,
}

impl Section {
    fn title(self) -> &'static str {
        match self {
            Self::Messages => "Messages",
            Self::Friends => "Friends",
            Self::Groups => "Groups",
            Self::Settings => "Settings",
        }
    }

    fn eyebrow(self) -> &'static str {
        match self {
            Self::Messages => "DIRECT MESSAGES",
            Self::Friends => "FRIENDS",
            Self::Groups => "YOUR GROUPS",
            Self::Settings => "PREFERENCES",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoginField {
    Server,
    AccessKey,
    RecoveryKey,
    Composer,
}

struct ImageViewer {
    images: Vec<EncryptedAttachmentMetadata>,
    index: usize,
    zoom: f32,
    loading: bool,
    error: Option<String>,
}

pub struct LarptrixShell {
    runtime: Handle,
    event_tx: Sender<AppEvent>,
    core: ClientCore,
    outgoing: Option<Sender<ClientMessage>>,
    crypto: Option<Arc<NativeMatrixCrypto>>,
    decrypted_messages: HashMap<(String, String), String>,
    decrypted_attachments: HashMap<(String, String), Vec<EncryptedAttachmentMetadata>>,
    cached_attachment_paths: HashMap<String, PathBuf>,
    media_cache_dir: Option<Arc<tempfile::TempDir>>,
    image_viewer: Option<ImageViewer>,
    pending_attachments: Vec<PickedFile>,
    pending_decryptions: HashMap<(String, String), ChatMessage>,
    decrypting_messages: HashSet<(String, String)>,
    server_url: String,
    access_key: String,
    recovery_key: String,
    composer_text: String,
    input_focus: FocusHandle,
    active_input: Option<LoginField>,
    connecting: bool,
    sending_message: bool,
    connection_status: String,
    active_section: Section,
    selected_peer: Option<String>,
}

impl LarptrixShell {
    pub fn new(cx: &mut Context<Self>, runtime: Handle) -> Self {
        let (event_tx, event_rx) = async_channel::bounded::<AppEvent>(256);

        cx.spawn(async move |this, cx| {
            while let Ok(event) = event_rx.recv().await {
                if this
                    .update(cx, |this, cx| this.handle_event(event, cx))
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();

        Self {
            runtime,
            event_tx,
            core: ClientCore::new(),
            outgoing: None,
            crypto: None,
            decrypted_messages: HashMap::new(),
            decrypted_attachments: HashMap::new(),
            cached_attachment_paths: HashMap::new(),
            media_cache_dir: tempfile::Builder::new()
                .prefix("larptrix-media-")
                .tempdir()
                .ok()
                .map(Arc::new),
            image_viewer: None,
            pending_attachments: Vec::new(),
            pending_decryptions: HashMap::new(),
            decrypting_messages: HashSet::new(),
            server_url: std::env::var("LARPTRIX_SERVER_URL").unwrap_or_default(),
            access_key: String::new(),
            recovery_key: String::new(),
            composer_text: String::new(),
            input_focus: cx.focus_handle(),
            active_input: Some(LoginField::Server),
            connecting: false,
            sending_message: false,
            connection_status: "Enter your server address and access key".into(),
            active_section: Section::Messages,
            selected_peer: None,
        }
    }

    fn handle_event(&mut self, event: AppEvent, cx: &mut Context<Self>) {
        match event {
            AppEvent::Authenticated { user, outgoing, crypto } => {
                self.connecting = false;
                self.access_key.clear();
                self.recovery_key.clear();
                self.outgoing = Some(outgoing);
                self.crypto = Some(crypto);
                self.core.apply_server_message(ServerMessage::Welcome {
                    user,
                    users: Vec::new(),
                });
                self.connection_status = "Signed in · synchronizing".into();
                self.active_section = Section::Messages;
            }
            AppEvent::Server(message) => {
                let decrypt_targets: Vec<(String, ChatMessage)> = match &message {
                    ServerMessage::Chat { peer, history } => history
                        .iter()
                        .cloned()
                        .map(|message| (peer.user_id.clone(), message))
                        .collect(),
                    ServerMessage::Message { message } => {
                        let current_user_id = self.core.state().current_user.as_ref()
                            .map(|user| user.user_id.as_str())
                            .unwrap_or_default();
                        let recipient_is_group = self.core.state().groups.contains_key(&message.recipient_id)
                            || self.core.state().users.get(&message.recipient_id)
                                .is_some_and(|user| user.is_group || user.is_channel);
                        let peer_id = if recipient_is_group || message.sender_id == current_user_id {
                            message.recipient_id.clone()
                        } else {
                            message.sender_id.clone()
                        };
                        vec![(peer_id, message.clone())]
                    }
                    _ => Vec::new(),
                };
                let is_welcome = matches!(&message, ServerMessage::Welcome { .. });
                let is_error = matches!(&message, ServerMessage::Error { .. });
                let suggested_peer = match &message {
                    ServerMessage::Welcome { user, users } => users
                        .iter()
                        .find(|candidate| candidate.user_id != user.user_id)
                        .map(|candidate| candidate.user_id.clone()),
                    ServerMessage::Directory { users } => self
                        .core
                        .state()
                        .current_user
                        .as_ref()
                        .and_then(|me| {
                            users
                                .iter()
                                .find(|candidate| candidate.user_id != me.user_id)
                        })
                        .map(|candidate| candidate.user_id.clone()),
                    _ => None,
                };

                self.core.apply_server_message(message);
                for (peer_id, message) in decrypt_targets {
                    self.queue_decryption(peer_id, message);
                }

                let peer_to_open = if is_welcome {
                    self.selected_peer.clone().or(suggested_peer)
                } else if self.selected_peer.is_none() {
                    suggested_peer
                } else {
                    None
                };
                if let Some(peer_id) = peer_to_open {
                    self.open_chat(peer_id);
                }

                if self.core.state().current_user.is_some() && !is_error {
                    self.connection_status = "Connected".into();
                }
            }
            AppEvent::Decrypted { peer_id, message_id, body, attachments } => {
                let key = (peer_id, message_id);
                self.decrypting_messages.remove(&key);
                self.pending_decryptions.remove(&key);
                self.decrypted_messages.insert(key.clone(), body);
                if !attachments.is_empty() {
                    self.decrypted_attachments.insert(key, attachments);
                }
            }
            AppEvent::ImageCached { attachment_id, path, error } => {
                if let Some(error) = error {
                    if let Some(viewer) = self.image_viewer.as_mut() {
                        if viewer.images.get(viewer.index).is_some_and(|image| image.id == attachment_id) {
                            viewer.loading = false;
                            viewer.error = Some(error.clone());
                        }
                    }
                    self.connection_status = format!("Image preview failed: {error}");
                } else {
                    self.cached_attachment_paths.insert(attachment_id.clone(), path);
                    if let Some(viewer) = self.image_viewer.as_mut() {
                        if viewer.images.get(viewer.index).is_some_and(|image| image.id == attachment_id) {
                            viewer.loading = false;
                            viewer.error = None;
                        }
                    }
                }
            }
            AppEvent::FilesPicked(files) => {
                let mut added = 0usize;
                for file in files {
                    if file.size_bytes == 0 {
                        continue;
                    }
                    if file.size_bytes > larptrix_protocol::MAX_ATTACHMENT_BYTES as u64 - 16 {
                        self.connection_status = format!("{} is larger than the 25 MiB encrypted upload limit", file.name);
                        continue;
                    }
                    if self.pending_attachments.iter().any(|item| item.path == file.path) {
                        continue;
                    }
                    if self.pending_attachments.len() >= 32 {
                        self.connection_status = "A message can contain at most 32 attachments".into();
                        break;
                    }
                    self.pending_attachments.push(file);
                    added += 1;
                }
                if added > 0 {
                    self.connection_status = format!("Added {added} attachment(s)");
                }
            }
            AppEvent::DecryptionFailed { peer_id, message, error } => {
                let key = (peer_id.clone(), message.id.clone());
                self.decrypting_messages.remove(&key);
                self.pending_decryptions.insert(key, message);
                self.connection_status = format!("Waiting for an E2E room key: {error}");
            }
            AppEvent::E2ESynced => {
                let pending = std::mem::take(&mut self.pending_decryptions);
                for ((peer_id, _), message) in pending {
                    self.queue_decryption(peer_id, message);
                }
            }
            AppEvent::Status(status) => {
                self.connection_status = status;
            }
            AppEvent::MessageSendFinished { body, success, error } => {
                self.sending_message = false;
                if success {
                    self.composer_text.clear();
                    self.pending_attachments.clear();
                    self.connection_status = "Encrypted message sent".into();
                } else {
                    self.composer_text = body;
                    self.connection_status = error.unwrap_or_else(|| "Message could not be sent".into());
                }
            }
            AppEvent::Error(error) => {
                self.connecting = false;
                self.connection_status = error;
            }
        }
        cx.notify();
    }

    fn start_connection(&mut self, cx: &mut Context<Self>) {
        if self.connecting || self.core.state().current_user.is_some() {
            return;
        }

        let server_url = self.server_url.trim().to_string();
        let entered_key = self.access_key.trim().to_string();
        let recovery_key = self.recovery_key.trim().to_string();
        let access_key: String = entered_key
            .chars()
            .filter(|ch| ch.is_ascii_hexdigit())
            .collect();
        let key_format_is_valid = entered_key
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() || ch == '-' || ch.is_ascii_whitespace());

        if server_url.is_empty() {
            self.connection_status = "Enter your Larptrix server URL".into();
            cx.notify();
            return;
        }
        if !key_format_is_valid || access_key.len() != 64 {
            self.connection_status =
                "Enter the 64-hex access key (with or without grouping dashes)".into();
            cx.notify();
            return;
        }
        if recovery_key.is_empty() {
            self.connection_status = "Enter the E2E recovery key from your existing web account".into();
            cx.notify();
            return;
        }

        self.connecting = true;
        self.connection_status = "Starting secure session…".into();
        cx.notify();

        let events: Sender<AppEvent> = Sender::clone(&self.event_tx);
        self.runtime.spawn(async move {
            if let Err(error) = network::connect(server_url, access_key, recovery_key, events.clone()).await {
                let _ = events.send_blocking(AppEvent::Error(error));
            }
        });
    }

    fn handle_input_key(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(field) = self.active_input else {
            return;
        };
        if self.sending_message && field == LoginField::Composer {
            return;
        }

        if event.keystroke.key == "backspace" {
            match field {
                LoginField::Server => { self.server_url.pop(); }
                LoginField::AccessKey => { self.access_key.pop(); }
                LoginField::RecoveryKey => { self.recovery_key.pop(); }
                LoginField::Composer => { self.composer_text.pop(); }
            }
            cx.notify();
            return;
        }

        if event.keystroke.key == "enter" {
            if field == LoginField::Composer {
                self.send_message(cx);
            } else {
                self.start_connection(cx);
            }
            return;
        }

        let is_paste = event.keystroke.key == "v"
            && (event.keystroke.modifiers.control || event.keystroke.modifiers.platform);
        if is_paste {
            if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                let current_len = match field {
                    LoginField::Server => self.server_url.len(),
                    LoginField::AccessKey => self.access_key.len(),
                    LoginField::RecoveryKey => self.recovery_key.len(),
                    LoginField::Composer => self.composer_text.len(),
                };
                let max_len = match field {
                    LoginField::Server => 256usize,
                    LoginField::AccessKey => 80usize,
                    LoginField::RecoveryKey => 80usize,
                    LoginField::Composer => 12_000usize,
                };
                let remaining = max_len.saturating_sub(current_len);
                let pasted: String = text.chars()
                    .filter(|ch| !ch.is_control())
                    .filter(|ch| match field {
                        LoginField::AccessKey => ch.is_ascii_hexdigit() || *ch == '-' || ch.is_ascii_whitespace(),
                        LoginField::RecoveryKey => ch.is_ascii_alphanumeric() || matches!(*ch, '+' | '/' | '=' ) || ch.is_ascii_whitespace(),
                        _ => true,
                    })
                    .take(remaining)
                    .collect();
                match field {
                    LoginField::Server => self.server_url.push_str(&pasted),
                    LoginField::AccessKey => self.access_key.push_str(&pasted),
                    LoginField::RecoveryKey => self.recovery_key.push_str(&pasted),
                    LoginField::Composer => self.composer_text.push_str(&pasted),
                }
                cx.notify();
            }
            return;
        }

        if event.keystroke.modifiers.control || event.keystroke.modifiers.platform {
            return;
        }

        if let Some(text) = event.keystroke.key_char.as_ref() {
            let (current_len, max_len) = match field {
                LoginField::Server => (self.server_url.len(), 256usize),
                LoginField::AccessKey => (self.access_key.len(), 80usize),
                LoginField::RecoveryKey => (self.recovery_key.len(), 80usize),
                LoginField::Composer => (self.composer_text.len(), 12_000usize),
            };
            if current_len + text.len() > max_len || text.chars().any(char::is_control) {
                return;
            }
            let valid = match field {
                LoginField::AccessKey => text.chars().all(|ch| ch.is_ascii_hexdigit() || ch == '-' || ch.is_ascii_whitespace()),
                LoginField::RecoveryKey => text.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '/' | '=' ) || ch.is_ascii_whitespace()),
                _ => true,
            };
            if !valid {
                return;
            }
            match field {
                LoginField::Server => self.server_url.push_str(text),
                LoginField::AccessKey => self.access_key.push_str(text),
                LoginField::RecoveryKey => self.recovery_key.push_str(text),
                LoginField::Composer => self.composer_text.push_str(text),
            }
            cx.notify();
        }
    }

    fn login_field(
        &self,
        field: LoginField,
        label: &'static str,
        placeholder: &'static str,
        value: &str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let focused = self.active_input == Some(field);
        let display_value = if matches!(field, LoginField::AccessKey | LoginField::RecoveryKey) && !value.is_empty() {
            "•".repeat(value.chars().count())
        } else {
            value.to_string()
        };
        let contents = if display_value.is_empty() {
            placeholder.to_string()
        } else {
            display_value
        };

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(MUTED))
                    .child(label),
            )
            .child(
                div()
                    .id(match field {
                        LoginField::Server => "server-url-input",
                        LoginField::AccessKey => "access-key-input",
                        LoginField::RecoveryKey => "recovery-key-input",
                        LoginField::Composer => "message-composer",
                    })
                    .track_focus(&self.input_focus)
                    .w_full()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(if focused { rgb(ACCENT) } else { rgb(BORDER) })
                    .bg(rgb(BG))
                    .cursor(gpui::CursorStyle::IBeam)
                    .text_sm()
                    .text_color(if contents == placeholder {
                        rgb(MUTED)
                    } else {
                        rgb(TEXT)
                    })
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, window, cx| {
                            this.active_input = Some(field);
                            window.focus(&this.input_focus, cx);
                            cx.notify();
                        }),
                    )
                    .on_key_down(cx.listener(Self::handle_input_key))
                    .child(contents),
            )
    }

    fn send_message(&mut self, cx: &mut Context<Self>) {
        let body = self.composer_text.trim().to_string();
        if (body.is_empty() && self.pending_attachments.is_empty())
            || self.connecting
            || self.sending_message
        {
            return;
        }
        let Some(peer_id) = self.selected_peer.clone() else {
            return;
        };
        let Some(crypto) = self.crypto.clone() else {
            self.connection_status = "E2E is not unlocked; message was not sent".into();
            cx.notify();
            return;
        };
        let Some(outgoing) = self.outgoing.clone() else {
            self.connection_status = "Not connected to the message transport".into();
            cx.notify();
            return;
        };

        let group_members = self
            .core
            .state()
            .groups
            .get(&peer_id)
            .map(|group| group.member_ids.clone());
        let files = self.pending_attachments.clone();

        self.sending_message = true;
        self.connection_status = if files.is_empty() {
            "Encrypting message…".into()
        } else {
            format!("Encrypting and uploading {} attachment(s)…", files.len())
        };
        cx.notify();

        let events: Sender<AppEvent> = Sender::clone(&self.event_tx);
        self.runtime.spawn(async move {
            let result = async {
                let mut uploaded_files = Vec::with_capacity(files.len());
                let mut attachment_ids = Vec::with_capacity(files.len());

                for file in files {
                    let bytes = tokio::fs::read(&file.path)
                        .await
                        .map_err(|error| format!("Could not read {}: {error}", file.name))?;
                    if bytes.len() as u64 != file.size_bytes {
                        return Err(format!("{} changed after it was selected; choose it again", file.name));
                    }
                    let mime = mime_guess::from_path(&file.path)
                        .first_or_octet_stream()
                        .essence_str()
                        .to_string();
                    let metadata = crypto
                        .encrypt_attachment_and_upload(&file.name, &mime, &bytes)
                        .await?;
                    attachment_ids.push(metadata.id.clone());
                    uploaded_files.push(serde_json::to_value(metadata)
                        .map_err(|error| format!("Could not encode attachment metadata: {error}"))?);
                }

                let mut payload = serde_json::json!({ "text": body });
                if !uploaded_files.is_empty() {
                    payload["files"] = serde_json::Value::Array(uploaded_files);
                }

                let encrypted_body = match group_members {
                    Some(member_ids) => crypto
                        .encrypt_group_payload(&peer_id, &member_ids, payload)
                        .await?,
                    None => crypto.encrypt_dm_payload(&peer_id, payload).await?,
                };

                outgoing
                    .send(ClientMessage::Send {
                        peer_id,
                        body: encrypted_body,
                        attachment_id: attachment_ids.first().cloned(),
                        attachment_ids,
                    })
                    .await
                    .map_err(|error| format!("Could not queue encrypted message: {error}"))?;

                Ok::<(), String>(())
            }.await;

            match result {
                Ok(()) => {
                    let _ = events.send_blocking(AppEvent::MessageSendFinished {
                        body,
                        success: true,
                        error: None,
                    });
                }
                Err(error) => {
                    let _ = events.send_blocking(AppEvent::MessageSendFinished {
                        body,
                        success: false,
                        error: Some(error),
                    });
                }
            }
        });
    }

    fn queue_decryption(&mut self, peer_id: String, message: ChatMessage) {
        let key = (peer_id.clone(), message.id.clone());
        if self.decrypted_messages.contains_key(&key) || !self.decrypting_messages.insert(key.clone()) {
            return;
        }

        let Ok(envelope) = serde_json::from_str::<serde_json::Value>(&message.body) else {
            self.decrypting_messages.remove(&key);
            return;
        };
        if envelope.get("version").and_then(serde_json::Value::as_i64) != Some(3)
            || envelope.get("message_type").and_then(serde_json::Value::as_str) != Some("matrix")
        {
            self.decrypting_messages.remove(&key);
            return;
        }
        let Some(crypto) = self.crypto.clone() else {
            self.decrypting_messages.remove(&key);
            return;
        };

        let events: Sender<AppEvent> = Sender::clone(&self.event_tx);
        let message_id = message.id.clone();
        let sender_id = message.sender_id.clone();
        let created_at = message.created_at;
        self.runtime.spawn(async move {
            match crypto.decrypt_message(&message_id, &sender_id, created_at, &envelope).await {
                Ok(payload) => {
                    let body = payload.get("text").and_then(serde_json::Value::as_str)
                        .map(str::to_string)
                        .unwrap_or_else(|| {
                            if payload.get("files").and_then(serde_json::Value::as_array).is_some() {
                                String::new()
                            } else {
                                payload.to_string()
                            }
                        });
                    let attachments = payload
                        .get("files")
                        .and_then(serde_json::Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(|item| serde_json::from_value::<EncryptedAttachmentMetadata>(item.clone()).ok())
                        .collect();
                    let _ = events.send_blocking(AppEvent::Decrypted {
                        peer_id,
                        message_id,
                        body,
                        attachments,
                    });
                }
                Err(error) => {
                    let _ = events.send_blocking(AppEvent::DecryptionFailed {
                        peer_id,
                        message,
                        error,
                    });
                }
            }
        });
    }

    fn open_chat(&mut self, peer_id: String) {
        self.selected_peer = Some(peer_id.clone());
        if let Some(outgoing) = &self.outgoing {
            let _ = outgoing.try_send(ClientMessage::Open { peer_id });
        }
    }

    fn toggle_reaction(
        &mut self,
        peer_id: String,
        message_id: String,
        emoji: String,
        cx: &mut Context<Self>,
    ) {
        let Some(outgoing) = self.outgoing.clone() else {
            self.connection_status = "Not connected to the message transport".into();
            cx.notify();
            return;
        };

        let add = self
            .core
            .state()
            .chats
            .get(&peer_id)
            .and_then(|messages| messages.iter().find(|message| message.id == message_id))
            .and_then(|message| {
                message
                    .reactions
                    .iter()
                    .find(|reaction| reaction.emoji == emoji)
            })
            .map(|reaction| !reaction.reacted)
            .unwrap_or(true);

        if outgoing
            .try_send(ClientMessage::React {
                peer_id,
                message_id,
                emoji,
                add,
            })
            .is_err()
        {
            self.connection_status = "Could not queue reaction · try again".into();
        } else {
            self.connection_status = "Updating reaction…".into();
        }
        cx.notify();
    }

    fn pick_attachments(&mut self, cx: &mut Context<Self>) {
        if self.sending_message {
            return;
        }
        let events = self.event_tx.clone();
        self.connection_status = "Choose files to attach…".into();
        cx.notify();

        self.runtime.spawn(async move {
            let selected = rfd::AsyncFileDialog::new()
                .set_title("Attach files to Larptrix")
                .pick_files()
                .await;

            let mut files = Vec::new();
            for file in selected.unwrap_or_default() {
                let path = file.path().to_path_buf();
                let size_bytes = match tokio::fs::metadata(&path).await {
                    Ok(metadata) => metadata.len(),
                    Err(error) => {
                        let _ = events.send_blocking(AppEvent::Error(format!(
                            "Could not read {}: {error}",
                            file.file_name()
                        )));
                        continue;
                    }
                };
                files.push(PickedFile {
                    path,
                    name: file.file_name(),
                    size_bytes,
                });
            }

            let _ = events.send_blocking(AppEvent::FilesPicked(files));
        });
    }

    fn remove_pending_attachment(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.pending_attachments.len() {
            self.pending_attachments.remove(index);
            self.connection_status = "Attachment removed".into();
            cx.notify();
        }
    }

    fn open_attachment(
        &mut self,
        peer_id: String,
        message_id: String,
        metadata: EncryptedAttachmentMetadata,
        cx: &mut Context<Self>,
    ) {
        if metadata.mime.starts_with("image/") {
            self.open_image_gallery(peer_id, message_id, metadata.id, cx);
        } else if metadata.mime.starts_with("video/") || metadata.mime.starts_with("audio/") {
            self.play_attachment_externally(metadata, cx);
        } else {
            self.save_attachment(metadata, cx);
        }
    }

    fn open_image_gallery(
        &mut self,
        peer_id: String,
        message_id: String,
        attachment_id: String,
        cx: &mut Context<Self>,
    ) {
        let images: Vec<_> = self
            .decrypted_attachments
            .get(&(peer_id.clone(), message_id.clone()))
            .into_iter()
            .flatten()
            .filter(|attachment| attachment.mime.starts_with("image/"))
            .cloned()
            .collect();

        if images.is_empty() {
            self.connection_status = "No image attachments are available in this message".into();
            cx.notify();
            return;
        }

        let index = images
            .iter()
            .position(|image| image.id == attachment_id)
            .unwrap_or(0);
        self.image_viewer = Some(ImageViewer {
            images,
            index,
            zoom: 1.0,
            loading: false,
            error: None,
        });
        self.load_viewer_image(cx);
        cx.notify();
    }

    fn load_viewer_image(&mut self, cx: &mut Context<Self>) {
        let Some(viewer) = self.image_viewer.as_mut() else {
            return;
        };
        let Some(metadata) = viewer.images.get(viewer.index).cloned() else {
            return;
        };
        if self.cached_attachment_paths.contains_key(&metadata.id) {
            viewer.loading = false;
            viewer.error = None;
            return;
        }
        let Some(crypto) = self.crypto.clone() else {
            viewer.loading = false;
            viewer.error = Some("E2E is locked".into());
            return;
        };
        let Some(cache_dir) = self.media_cache_dir.clone() else {
            viewer.loading = false;
            viewer.error = Some("Could not create a private temporary media directory".into());
            return;
        };

        viewer.loading = true;
        viewer.error = None;
        let events = self.event_tx.clone();
        self.runtime.spawn(async move {
            let result = async {
                let bytes = crypto.download_and_decrypt_attachment(&metadata).await?;
                let extension = media_extension(&metadata.mime);
                let path = cache_dir.path().join(format!("{}.{}", uuid::Uuid::new_v4(), extension));
                tokio::fs::write(&path, bytes)
                    .await
                    .map_err(|error| format!("Could not cache decrypted image: {error}"))?;
                Ok::<PathBuf, String>(path)
            }.await;

            let event = match result {
                Ok(path) => AppEvent::ImageCached {
                    attachment_id: metadata.id,
                    path,
                    error: None,
                },
                Err(error) => AppEvent::ImageCached {
                    attachment_id: metadata.id,
                    path: PathBuf::new(),
                    error: Some(error),
                },
            };
            let _ = events.send_blocking(event);
        });
    }

    fn navigate_image_gallery(&mut self, delta: isize, cx: &mut Context<Self>) {
        let Some(viewer) = self.image_viewer.as_mut() else {
            return;
        };
        let len = viewer.images.len();
        if len < 2 {
            return;
        }
        viewer.index = (viewer.index as isize + delta).rem_euclid(len as isize) as usize;
        viewer.loading = false;
        viewer.error = None;
        self.load_viewer_image(cx);
        cx.notify();
    }

    fn adjust_image_zoom(&mut self, delta: f32, cx: &mut Context<Self>) {
        if let Some(viewer) = self.image_viewer.as_mut() {
            viewer.zoom = (viewer.zoom + delta).clamp(0.5, 3.0);
            cx.notify();
        }
    }

    fn close_image_viewer(&mut self, cx: &mut Context<Self>) {
        self.image_viewer = None;
        cx.notify();
    }

    fn play_attachment_externally(
        &mut self,
        metadata: EncryptedAttachmentMetadata,
        cx: &mut Context<Self>,
    ) {
        let Some(crypto) = self.crypto.clone() else {
            self.connection_status = "E2E is locked; media cannot be decrypted".into();
            cx.notify();
            return;
        };
        let Some(cache_dir) = self.media_cache_dir.clone() else {
            self.connection_status = "Could not create a private temporary media directory".into();
            cx.notify();
            return;
        };

        self.connection_status = format!("Preparing {}…", metadata.name);
        cx.notify();
        let events = self.event_tx.clone();
        self.runtime.spawn(async move {
            let result = async {
                let bytes = crypto.download_and_decrypt_attachment(&metadata).await?;
                let path = cache_dir.path().join(format!(
                    "{}.{}",
                    uuid::Uuid::new_v4(),
                    media_extension(&metadata.mime),
                ));
                tokio::fs::write(&path, bytes)
                    .await
                    .map_err(|error| format!("Could not cache decrypted media: {error}"))?;
                open::that(&path)
                    .map_err(|error| format!("Could not launch the system media player: {error}"))?;
                Ok::<(), String>(())
            }.await;

            match result {
                Ok(()) => {
                    let _ = events.send_blocking(AppEvent::Status(format!(
                        "Opened {} in the system media player",
                        metadata.name
                    )));
                }
                Err(error) => {
                    let _ = events.send_blocking(AppEvent::Error(error));
                }
            }
        });
    }

    fn save_attachment(
        &mut self,
        metadata: EncryptedAttachmentMetadata,
        cx: &mut Context<Self>,
    ) {
        let Some(crypto) = self.crypto.clone() else {
            self.connection_status = "E2E is locked; attachment cannot be decrypted".into();
            cx.notify();
            return;
        };

        self.connection_status = format!("Downloading and decrypting {}…", metadata.name);
        cx.notify();
        let events = self.event_tx.clone();
        self.runtime.spawn(async move {
            let result = async {
                let bytes = crypto.download_and_decrypt_attachment(&metadata).await?;
                let file_name = std::path::Path::new(&metadata.name)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .filter(|name| !name.is_empty())
                    .unwrap_or("attachment")
                    .to_string();
                let Some(destination) = rfd::AsyncFileDialog::new()
                    .set_title("Save decrypted attachment")
                    .set_file_name(file_name)
                    .save_file()
                    .await
                else {
                    return Ok::<String, String>("Save cancelled".into());
                };

                destination.write(&bytes).await
                    .map_err(|error| format!("Could not save decrypted attachment: {error}"))?;
                Ok(format!("Saved {}", metadata.name))
            }.await;

            match result {
                Ok(status) => {
                    let _ = events.send_blocking(AppEvent::Status(status));
                }
                Err(error) => {
                    let _ = events.send_blocking(AppEvent::Error(error));
                }
            }
        });
    }

    fn server_rail(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(72.0))
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .gap_3()
            .py_4()
            .bg(rgb(RAIL))
            .border_r_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .size(px(44.0))
                    .rounded_xl()
                    .bg(rgb(ACCENT))
                    .text_color(rgb(RAIL))
                    .font_weight(gpui::FontWeight::BOLD)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_xl()
                    .child("L"),
            )
            .child(div().w(px(30.0)).h(px(1.0)).bg(rgb(BORDER)))
            .child(
                rail_button(
                    "rail-messages",
                    "◈",
                    self.active_section == Section::Messages,
                    cx.listener(|this, _, _, cx| {
                        this.active_section = Section::Messages;
                        cx.notify();
                    }),
                ),
            )
            .child(
                rail_button(
                    "rail-friends",
                    "♙",
                    self.active_section == Section::Friends,
                    cx.listener(|this, _, _, cx| {
                        this.active_section = Section::Friends;
                        cx.notify();
                    }),
                ),
            )
            .child(
                rail_button(
                    "rail-groups",
                    "▦",
                    self.active_section == Section::Groups,
                    cx.listener(|this, _, _, cx| {
                        this.active_section = Section::Groups;
                        cx.notify();
                    }),
                ),
            )
            .child(div().flex_1())
            .child(
                rail_button(
                    "rail-settings",
                    "⚙",
                    self.active_section == Section::Settings,
                    cx.listener(|this, _, _, cx| {
                        this.active_section = Section::Settings;
                        cx.notify();
                    }),
                ),
            )
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match self.active_section {
            Section::Messages => self.message_list(cx).into_any_element(),
            Section::Friends => self.friends_list().into_any_element(),
            Section::Groups => self.groups_list(cx).into_any_element(),
            Section::Settings => self.settings_list().into_any_element(),
        };

        let (account_name, username) = self
            .core
            .state()
            .current_user
            .as_ref()
            .map(|user| (user.display_name.clone(), user.username.clone()))
            .unwrap_or_else(|| ("Account".into(), String::new()));

        div()
            .w(px(264.0))
            .h_full()
            .flex()
            .flex_col()
            .bg(rgb(SIDEBAR))
            .border_r_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .h(px(64.0))
                    .px_5()
                    .flex()
                    .items_center()
                    .border_b_1()
                    .border_color(rgb(BORDER))
                    .child(
                        div()
                            .text_base()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(TEXT))
                            .child(self.active_section.title()),
                    ),
            )
            .child(
                div()
                    .px_4()
                    .pt_5()
                    .pb_2()
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(MUTED))
                    .child(self.active_section.eyebrow()),
            )
            .child(content)
            .child(
                div()
                    .mt_auto()
                    .p_3()
                    .border_t_1()
                    .border_color(rgb(BORDER))
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .size(px(36.0))
                            .rounded_full()
                            .bg(rgb(SURFACE_HOVER))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_sm()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(ACCENT))
                            .child(account_name.chars().next().unwrap_or('U').to_string()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(rgb(TEXT))
                                    .child(account_name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(MUTED))
                                    .child(if username.is_empty() {
                                        self.connection_status.clone()
                                    } else {
                                        format!("@{username}")
                                    }),
                            ),
                    )
                    .child(div().size(px(7.0)).rounded_full().bg(rgb(GREEN))),
            )
    }

    fn message_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let me_id = self
            .core
            .state()
            .current_user
            .as_ref()
            .map(|user| user.user_id.as_str())
            .unwrap_or_default();

        let rows: Vec<_> = self
            .core
            .state()
            .users
            .values()
            .filter(|user| user.user_id != me_id && !user.is_group && !user.is_channel)
            .map(|user| {
                let peer_id = user.user_id.clone();
                let name = user.display_name.clone();
                let online = user.online;
                let preview = self
                    .core
                    .state()
                    .chats
                    .get(&peer_id)
                    .and_then(|messages| messages.last())
                    .map(|_| "Encrypted message".to_string())
                    .unwrap_or_else(|| {
                        if online {
                            "Start a conversation".into()
                        } else {
                            "Offline".into()
                        }
                    });
                (peer_id, name, preview, online)
            })
            .collect();
        let no_peers = rows.is_empty();

        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_1()
            .px_2()
            .children(rows.into_iter().map(|(peer_id, name, preview, online)| {
                let selected = self.selected_peer.as_deref() == Some(peer_id.as_str());
                let click_id = peer_id.clone();
                chat_entry(
                    peer_id,
                    name,
                    preview,
                    if online { "ONLINE" } else { "OFFLINE" }.into(),
                    selected,
                    cx.listener(move |this, _, _, cx| {
                        this.active_section = Section::Messages;
                        this.open_chat(click_id.clone());
                        cx.notify();
                    }),
                )
            }))
            .when(no_peers, |element| {
                element.child(empty_hint("No users in the server directory yet"))
            })
    }

    fn friends_list(&self) -> impl IntoElement {
        let me_id = self
            .core
            .state()
            .current_user
            .as_ref()
            .map(|user| user.user_id.as_str())
            .unwrap_or_default();
        let users: Vec<_> = self
            .core
            .state()
            .users
            .values()
            .filter(|user| user.user_id != me_id && !user.is_group && !user.is_channel)
            .map(|user| (user.display_name.clone(), user.username.clone(), user.online))
            .collect();

        let no_users = users.is_empty();

        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .child(section_heading("PEOPLE ON THIS SERVER"))
            .children(users.into_iter().map(|(name, username, online)| {
                person_row(
                    name.chars().next().unwrap_or('?').to_string(),
                    name,
                    if online {
                        "Online".into()
                    } else if username.is_empty() {
                        "Offline".into()
                    } else {
                        format!("@{username}")
                    },
                )
            }))
            .when(no_users, |element| {
                element.child(empty_hint("No other users were returned by the server"))
            })
    }

    fn groups_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let groups: Vec<_> = self
            .core
            .state()
            .groups
            .values()
            .map(|group| {
                (
                    group.group_id.clone(),
                    group.name.clone(),
                    group.member_ids.len(),
                    group.is_channel,
                    group.description.clone(),
                )
            })
            .collect();

        let no_groups = groups.is_empty();

        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .children(groups.into_iter().map(|(group_id, name, members, is_channel, description)| {
                let click_id = group_id.clone();
                div()
                    .id(format!("group-{group_id}"))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.active_section = Section::Messages;
                        this.open_chat(click_id.clone());
                        cx.notify();
                    }))
                    .child(group_row(
                        if is_channel { "▤" } else { "⌘" }.into(),
                        name,
                        format!("{members} members"),
                        description,
                    ))
            }))
            .when(no_groups, |element| {
                element.child(empty_hint("No groups or channels yet"))
            })
    }

    fn settings_list(&self) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .child(settings_row("Connection", self.connection_status.clone()))
            .child(settings_row("Account", "Account profile and active sessions".to_string()))
            .child(settings_row(
                "End-to-end encryption",
                self.crypto.as_ref()
                    .map(|crypto| format!("Enabled · Matrix device {}", crypto.device_id()))
                    .unwrap_or_else(|| "Locked · sign in with your E2E recovery key".to_string()),
            ))
            .child(settings_row(
                "Appearance",
                "Native theme and layout preferences".to_string(),
            ))
    }

    fn conversation(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(peer_id) = self.selected_peer.as_deref() else {
            return div()
                .flex_1()
                .h_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .bg(rgb(BG))
                .gap_2()
                .child(
                    div()
                        .text_lg()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(TEXT))
                        .child("Your conversations"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(MUTED))
                        .child("Choose a person from the sidebar to load chat history"),
                )
                .child(
                    div()
                        .pt_4()
                        .text_xs()
                        .text_color(rgb(MUTED))
                        .child(self.connection_status.clone()),
                );
        };

        let Some(peer) = self.core.state().users.get(peer_id) else {
            return div()
                .flex_1()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(rgb(BG))
                .text_color(rgb(MUTED))
                .child("Loading conversation…");
        };

        let current_user_id = self
            .core
            .state()
            .current_user
            .as_ref()
            .map(|user| user.user_id.as_str())
            .unwrap_or_default();

        let messages: Vec<_> = self
            .core
            .state()
            .chats
            .get(peer_id)
            .into_iter()
            .flatten()
            .map(|message| {
                let mine = message.sender_id == current_user_id;
                let author = if mine {
                    "You".to_string()
                } else {
                    message.sender_name.clone()
                };
                let body = self.decrypted_messages
                    .get(&(peer_id.to_string(), message.id.clone()))
                    .cloned()
                    .unwrap_or_else(|| {
                        let looks_encrypted = serde_json::from_str::<serde_json::Value>(&message.body).is_ok();
                        if looks_encrypted {
                            "Encrypted message · decrypting…".to_string()
                        } else {
                            message.body.clone()
                        }
                    });
                (
                    message.id.clone(),
                    author,
                    body,
                    mine,
                    message.reactions.clone(),
                    self.decrypted_attachments
                        .get(&(peer_id.to_string(), message.id.clone()))
                        .cloned()
                        .unwrap_or_default(),
                )
            })
            .collect();

        let display_name = peer.display_name.clone();
        let handle = if peer.username.is_empty() {
            String::new()
        } else {
            format!("@{}", peer.username)
        };
        let status = if peer.online { "Online" } else { "Offline" };
        let pending_attachment_labels: Vec<(usize, String)> = self
            .pending_attachments
            .iter()
            .enumerate()
            .map(|(index, file)| (index, format!("{} · {}", file.name, format_attachment_size(file.size_bytes))))
            .collect();

        div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .bg(rgb(BG))
            .child(
                div()
                    .h(px(64.0))
                    .px_6()
                    .flex()
                    .items_center()
                    .gap_3()
                    .border_b_1()
                    .border_color(rgb(BORDER))
                    .child(
                        div()
                            .size(px(36.0))
                            .rounded_full()
                            .bg(rgb(SURFACE_HOVER))
                            .flex()
                            .items_center()
                            .justify_center()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(ACCENT))
                            .child(display_name.chars().next().unwrap_or('?').to_string()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(TEXT))
                                    .child(display_name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(MUTED))
                                    .child(status),
                            ),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(MUTED))
                            .child(handle.clone()),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .justify_end()
                    .gap_4()
                    .p_6()
                    .children(messages.iter().map(|(message_id, author, body, mine, reactions, attachments)| {
                        let reaction_peer_id = peer_id.to_string();
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .when(!body.is_empty(), |element| {
                                element.child(message_bubble(author.clone(), body.clone(), *mine))
                            })
                            .children(attachments.iter().map(|metadata| {
                                let metadata = metadata.clone();
                                let target_peer = reaction_peer_id.clone();
                                let target_message = message_id.clone();
                                let (icon, action) = if metadata.mime.starts_with("image/") {
                                    ("▧", "View")
                                } else if metadata.mime.starts_with("video/") {
                                    ("▶", "Play")
                                } else if metadata.mime.starts_with("audio/") {
                                    ("♫", "Play")
                                } else {
                                    ("⇩", "Save")
                                };
                                let label = format!(
                                    "{icon}  {}  ·  {}  ·  {}",
                                    action,
                                    metadata.name,
                                    format_attachment_size(metadata.size),
                                );
                                div()
                                    .id(format!("attachment-download-{}-{}", message_id, metadata.id))
                                    .px_3()
                                    .py_2()
                                    .rounded_md()
                                    .bg(rgb(SIDEBAR))
                                    .border_1()
                                    .border_color(rgb(BORDER))
                                    .text_xs()
                                    .text_color(rgb(ACCENT))
                                    .cursor_pointer()
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.open_attachment(
                                            target_peer.clone(),
                                            target_message.clone(),
                                            metadata.clone(),
                                            cx,
                                        );
                                    }))
                                    .child(label)
                            }))
                            .child(
                                div()
                                    .flex()
                                    .gap_1()
                                    .children(["👍", "❤️", "😂", "🔥"].into_iter().map(|emoji| {
                                        let count = reactions
                                            .iter()
                                            .find(|reaction| reaction.emoji == emoji)
                                            .map(|reaction| reaction.count)
                                            .unwrap_or(0);
                                        let reacted = reactions
                                            .iter()
                                            .any(|reaction| reaction.emoji == emoji && reaction.reacted);
                                        let label = if count == 0 {
                                            emoji.to_string()
                                        } else {
                                            format!("{emoji} {count}")
                                        };
                                        let peer_id = reaction_peer_id.clone();
                                        let message_id = message_id.clone();
                                        let emoji = emoji.to_string();

                                        div()
                                            .id(format!("reaction-{message_id}-{emoji}"))
                                            .px_2()
                                            .py_1()
                                            .rounded_md()
                                            .border_1()
                                            .border_color(if reacted { rgb(ACCENT) } else { rgb(BORDER) })
                                            .bg(if reacted { rgb(0x35495b) } else { rgb(SIDEBAR) })
                                            .text_xs()
                                            .text_color(if reacted { rgb(TEXT) } else { rgb(MUTED) })
                                            .cursor_pointer()
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.toggle_reaction(
                                                    peer_id.clone(),
                                                    message_id.clone(),
                                                    emoji.clone(),
                                                    cx,
                                                );
                                            }))
                                            .child(label)
                                    })),
                            )
                    }))
                    .when(messages.is_empty(), |element| {
                        element.child(empty_hint("No message history in this conversation"))
                    }),
            )
            .child(
                div()
                    .mx_5()
                    .mb_3()
                    .p_3()
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb(BORDER))
                    .bg(rgb(SIDEBAR))
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .id("attach-file-button")
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(SURFACE))
                            .border_1()
                            .border_color(rgb(BORDER))
                            .text_color(rgb(ACCENT))
                            .text_sm()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| this.pick_attachments(cx)))
                            .child("＋"),
                    )
                    .child(
                        div()
                            .id("message-composer")
                            .track_focus(&self.input_focus)
                            .flex_1()
                            .min_w(px(0.0))
                            .p_2()
                            .rounded_md()
                            .cursor(gpui::CursorStyle::IBeam)
                            .text_sm()
                            .text_color(if self.composer_text.is_empty() { rgb(MUTED) } else { rgb(TEXT) })
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                                this.active_input = Some(LoginField::Composer);
                                window.focus(&this.input_focus, cx);
                                cx.notify();
                            }))
                            .on_key_down(cx.listener(Self::handle_input_key))
                            .child(if self.composer_text.is_empty() {
                                "Write an encrypted message…".to_string()
                            } else {
                                self.composer_text.clone()
                            }),
                    )
                    .child(
                        div()
                            .id("send-message-button")
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(if self.sending_message { rgb(SURFACE_HOVER) } else { rgb(ACCENT) })
                            .text_color(rgb(RAIL))
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| this.send_message(cx)))
                            .child(if self.sending_message { "Sending…" } else { "Send" }),
                    ),
            )
            .when(!pending_attachment_labels.is_empty(), |element| {
                element.child(
                    div()
                        .mx_5()
                        .mb_2()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(pending_attachment_labels.iter().map(|(index, label)| {
                            let index = *index;
                            div()
                                .id(format!("pending-attachment-{index}"))
                                .flex()
                                .items_center()
                                .justify_between()
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(rgb(SURFACE))
                                .text_xs()
                                .text_color(rgb(MUTED))
                                .cursor_pointer()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.remove_pending_attachment(index, cx);
                                }))
                                .child(format!("{label}   ×"))
                        })),
                )
            })
            .child(
                div()
                    .px_5()
                    .pb_2()
                    .text_xs()
                    .text_color(rgb(MUTED))
                    .flex()
                    .justify_between()
                    .child(self.connection_status.clone())
                    .child(if handle.is_empty() {
                        String::new()
                    } else {
                        handle
                    }),
            )
    }
}

impl Render for LarptrixShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.core.state().current_user.is_none() {
            self.login_view(cx).into_any_element()
        } else {
            div()
                .size_full()
                .flex()
                .flex_row()
                .bg(rgb(BG))
                .text_color(rgb(TEXT))
                .relative()
                .child(self.server_rail(cx))
                .child(self.sidebar(cx))
                .child(self.conversation(cx))
                .when(self.image_viewer.is_some(), |element| {
                    element.child(self.image_viewer_overlay(cx))
                })
                .into_any_element()
        }
    }
}

impl LarptrixShell {
    fn image_viewer_overlay(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(viewer) = self.image_viewer.as_ref() else {
            return div().into_any_element();
        };
        let current = viewer.images.get(viewer.index);
        let title = current
            .map(|image| image.name.clone())
            .unwrap_or_else(|| "Image preview".into());
        let count = format!("{} / {}", viewer.index + 1, viewer.images.len());
        let path = current.and_then(|image| self.cached_attachment_paths.get(&image.id)).cloned();
        let has_path = path.is_some();
        let loading = viewer.loading;
        let error = viewer.error.clone();
        let has_error = error.is_some();
        let error_message = error.unwrap_or_default();
        let can_navigate = viewer.images.len() > 1;
        let zoom = viewer.zoom;

        div()
            .absolute()
            .inset_0()
            .occlude()
            .bg(rgb(0x10141d))
            .flex()
            .flex_col()
            .p_4()
            .gap_3()
            .child(
                div()
                    .h(px(48.0))
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(TEXT))
                            .child(title),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(MUTED))
                            .child(count),
                    )
                    .child(
                        div()
                            .id("close-image-viewer")
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(SURFACE))
                            .text_sm()
                            .text_color(rgb(TEXT))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.close_image_viewer(cx);
                            }))
                            .child("Close  ×"),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .overflow_hidden()
                    .when_some(path, |element, path| {
                        element.child(
                            img(path)
                                .w(px(960.0 * zoom))
                                .h(px(760.0 * zoom))
                                .object_fit(gpui::ObjectFit::Contain),
                        )
                    })
                    .when(loading, |element| {
                        element.child(
                            div()
                                .text_sm()
                                .text_color(rgb(MUTED))
                                .child("Decrypting image…"),
                        )
                    })
                    .when(has_error, |element| {
                        element.child(
                            div()
                                .text_sm()
                                .text_color(rgb(RED))
                                .child(error_message.clone()),
                        )
                    })
                    .when(!has_path && !loading && !has_error, |element| {
                        element.child(
                            div()
                                .text_sm()
                                .text_color(rgb(MUTED))
                                .child("Image is not available"),
                        )
                    }),
            )
            .child(
                div()
                    .h(px(48.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .child(
                        div()
                            .id("image-zoom-out")
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(SURFACE))
                            .text_color(rgb(TEXT))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.adjust_image_zoom(-0.25, cx);
                            }))
                            .child("−"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(MUTED))
                            .child(format!("{:.0}%", zoom * 100.0)),
                    )
                    .child(
                        div()
                            .id("image-zoom-in")
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(SURFACE))
                            .text_color(rgb(TEXT))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.adjust_image_zoom(0.25, cx);
                            }))
                            .child("+"),
                    )
                    .when(can_navigate, |element| {
                        element
                            .child(
                                div()
                                    .id("image-gallery-previous")
                                    .px_4()
                                    .py_2()
                                    .rounded_md()
                                    .bg(rgb(SURFACE))
                                    .text_color(rgb(TEXT))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.navigate_image_gallery(-1, cx);
                                    }))
                                    .child("‹  Previous"),
                            )
                            .child(
                                div()
                                    .id("image-gallery-next")
                                    .px_4()
                                    .py_2()
                                    .rounded_md()
                                    .bg(rgb(SURFACE))
                                    .text_color(rgb(TEXT))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.navigate_image_gallery(1, cx);
                                    }))
                                    .child("Next  ›"),
                            )
                    }),
            )
            .into_any_element()
    }

    fn login_view(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(BG))
            .child(
                div()
                    .w(px(460.0))
                    .p_6()
                    .rounded_xl()
                    .border_1()
                    .border_color(rgb(BORDER))
                    .bg(rgb(SIDEBAR))
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .size(px(48.0))
                            .rounded_xl()
                            .bg(rgb(ACCENT))
                            .text_color(rgb(RAIL))
                            .text_xl()
                            .font_weight(gpui::FontWeight::BOLD)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child("L"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(TEXT))
                                    .child("Welcome to Larptrix"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(MUTED))
                                    .child("Connect to your self-hosted server"),
                            ),
                    )
                    .child(self.login_field(
                        LoginField::Server,
                        "SERVER ADDRESS",
                        "https://chat.example.com",
                        &self.server_url,
                        cx,
                    ))
                    .child(self.login_field(
                        LoginField::AccessKey,
                        "ACCESS KEY",
                        "64-character access key",
                        &self.access_key,
                        cx,
                    ))
                    .child(self.login_field(
                        LoginField::RecoveryKey,
                        "E2E RECOVERY KEY",
                        "Base64 recovery key from web setup",
                        &self.recovery_key,
                        cx,
                    ))
                    .child(
                        div()
                            .id("connect-button")
                            .w_full()
                            .p_3()
                            .rounded_md()
                            .bg(rgb(ACCENT))
                            .text_color(rgb(RAIL))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.start_connection(cx);
                            }))
                            .child(if self.connecting {
                                "Connecting…"
                            } else {
                                "Connect securely"
                            }),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(if self.connection_status.contains("failed")
                                || self.connection_status.contains("invalid")
                                || self.connection_status.contains("must")
                                || self.connection_status.contains("failed")
                            {
                                rgb(RED)
                            } else {
                                rgb(MUTED)
                            })
                            .child(self.connection_status.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(MUTED))
                            .child(
                                "Use the E2E recovery key from your existing web account. It unlocks your encrypted device store and is never saved as plaintext.",
                            ),
                    ),
            )
    }
}

fn rail_button(
    id: &'static str,
    icon: &'static str,
    selected: bool,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .size(px(44.0))
        .rounded_xl()
        .bg(if selected {
            rgb(SURFACE_HOVER)
        } else {
            rgb(SURFACE)
        })
        .text_color(rgb(TEXT))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .on_click(on_click)
        .child(icon)
}

fn chat_entry(
    id: String,
    name: String,
    preview: String,
    time: String,
    selected: bool,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let initials = name.chars().next().unwrap_or('?').to_string();

    div()
        .id(id)
        .px_3()
        .py_3()
        .rounded_lg()
        .flex()
        .items_center()
        .gap_3()
        .cursor_pointer()
        .bg(if selected {
            rgb(SURFACE_HOVER)
        } else {
            rgb(SIDEBAR)
        })
        .hover(|style| style.bg(rgb(SURFACE)))
        .on_click(on_click)
        .child(
            div()
                .size(px(40.0))
                .rounded_full()
                .bg(rgb(SURFACE_HOVER))
                .flex()
                .items_center()
                .justify_center()
                .text_sm()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(ACCENT))
                .child(initials),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .flex_col()
                .gap_0p5()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(rgb(TEXT))
                                .child(name),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(MUTED))
                                .child(time),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(MUTED))
                        .child(preview),
                ),
        )
}

fn media_extension(mime: &str) -> &'static str {
    match mime {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/bmp" => "bmp",
        "image/avif" => "avif",
        "video/mp4" => "mp4",
        "video/webm" | "audio/webm" => "webm",
        "video/quicktime" => "mov",
        "video/x-matroska" => "mkv",
        "video/x-msvideo" => "avi",
        "video/x-ms-wmv" => "wmv",
        "audio/mpeg" => "mp3",
        "audio/mp4" => "m4a",
        "audio/ogg" | "audio/opus" => "ogg",
        "audio/aac" => "aac",
        "audio/wav" | "audio/x-wav" => "wav",
        "audio/flac" => "flac",
        _ => "bin",
    }
}

fn format_attachment_size(size: u64) -> String {
    if size < 1024 {
        format!("{size} B")
    } else if size < 1024 * 1024 {
        format!("{:.1} KiB", size as f64 / 1024.0)
    } else {
        format!("{:.1} MiB", size as f64 / (1024.0 * 1024.0))
    }
}

fn message_bubble(author: String, body: String, mine: bool) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_end()
        .when(!mine, |element| element.items_start())
        .gap_1()
        .child(
            div()
                .text_xs()
                .text_color(rgb(MUTED))
                .child(author),
        )
        .child(
            div()
                .max_w(px(560.0))
                .px_4()
                .py_3()
                .rounded_lg()
                .bg(if mine {
                    rgb(0x35495b)
                } else {
                    rgb(SURFACE)
                })
                .text_sm()
                .text_color(rgb(TEXT))
                .child(body),
        )
}

fn section_heading(text: &'static str) -> impl IntoElement {
    div()
        .pt_2()
        .pb_1()
        .text_xs()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(MUTED))
        .child(text)
}

fn empty_hint(text: &'static str) -> impl IntoElement {
    div()
        .p_3()
        .text_xs()
        .text_color(rgb(MUTED))
        .child(text)
}

fn person_row(initial: String, name: String, status: String) -> impl IntoElement {
    div()
        .p_2()
        .rounded_md()
        .flex()
        .items_center()
        .gap_3()
        .child(
            div()
                .size(px(34.0))
                .rounded_full()
                .bg(rgb(SURFACE_HOVER))
                .flex()
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(rgb(ACCENT))
                .child(initial),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(div().text_sm().text_color(rgb(TEXT)).child(name))
                .child(div().text_xs().text_color(rgb(MUTED)).child(status)),
        )
}

fn group_row(
    icon: String,
    name: String,
    members: String,
    description: String,
) -> impl IntoElement {
    div()
        .p_2()
        .rounded_md()
        .flex()
        .items_center()
        .gap_3()
        .child(
            div()
                .size(px(34.0))
                .rounded_lg()
                .bg(rgb(SURFACE_HOVER))
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(ACCENT))
                .child(icon),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(div().text_sm().text_color(rgb(TEXT)).child(name))
                .child(div().text_xs().text_color(rgb(MUTED)).child(members))
                .when(!description.is_empty(), |element| {
                    element.child(div().text_xs().text_color(rgb(MUTED)).child(description))
                }),
        )
}

fn settings_row(title: &'static str, subtitle: String) -> impl IntoElement {
    div()
        .p_3()
        .rounded_md()
        .bg(rgb(SURFACE))
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(rgb(TEXT))
                .child(title),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(MUTED))
                .child(subtitle),
        )
}

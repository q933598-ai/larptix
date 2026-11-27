use async_channel::Sender;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use gpui::{
    div, img, prelude::*, rgb, px, App, ClipboardItem, Context, FocusHandle, KeyDownEvent, MouseButton, Window,
};
use larptrix_client_core::ClientCore;
use larptrix_protocol::{ChatMessage, ClientMessage, ServerMessage, UserInfo};
use qrcode::{Color as QrColor, QrCode};
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;

use crate::crypto::{EncryptedAttachmentMetadata, NativeMatrixCrypto};
use crate::media::MediaPlayback;
use crate::network::{self, AppEvent, NativeApi, PickedFile};

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
    Saved,
    Friends,
    Groups,
    Settings,
}

impl Section {
    fn title(self) -> &'static str {
        match self {
            Self::Messages => "Messages",
            Self::Saved => "Saved Messages",
            Self::Friends => "Friends",
            Self::Groups => "Groups",
            Self::Settings => "Settings",
        }
    }

    fn eyebrow(self) -> &'static str {
        match self {
            Self::Messages => "DIRECT MESSAGES",
            Self::Saved => "PRIVATE NOTES",
            Self::Friends => "FRIENDS",
            Self::Groups => "YOUR GROUPS",
            Self::Settings => "PREFERENCES",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum AuthMode {
    #[default]
    SignIn,
    Register,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ConnectionPage {
    #[default]
    Servers,
    SignIn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum LoginMode {
    #[default]
    AccessKey,
    EmailPassword,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroupCreationMode {
    Group,
    Channel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoginField {
    Server,
    DisplayName,
    AccessKey,
    Email,
    Password,
    RecoveryKey,
    Composer,
    PeopleSearch,
    GroupName,
    ProfileDisplayName,
    ProfileUsername,
    ProfileAbout,
    SavedComposer,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SavedMessage {
    id: String,
    body: String,
    created_at: i64,
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
    loading_attachment_ids: HashSet<String>,
    cached_avatar_paths: HashMap<String, PathBuf>,
    loading_avatar_ids: HashSet<String>,
    media_cache_dir: Option<Arc<tempfile::TempDir>>,
    image_viewer: Option<ImageViewer>,
    media_player: Option<Arc<MediaPlayback>>,
    media_session_id: Option<String>,
    media_title: String,
    media_is_video: bool,
    media_path: Option<PathBuf>,
    media_frame: Option<Arc<gpui::RenderImage>>,
    media_position_ms: u64,
    media_duration_ms: u64,
    media_volume: f32,
    media_playing: bool,
    media_finished: bool,
    media_loading: bool,
    media_error: Option<String>,
    pending_attachments: Vec<PickedFile>,
    pending_decryptions: HashMap<(String, String), ChatMessage>,
    decrypting_messages: HashSet<(String, String)>,
    server_url: String,
    saved_servers: Vec<String>,
    api: Option<Arc<NativeApi>>,
    friends: Vec<UserInfo>,
    incoming_friend_requests: Vec<UserInfo>,
    outgoing_friend_requests: Vec<UserInfo>,
    user_search: String,
    user_search_results: Vec<UserInfo>,
    auth_mode: AuthMode,
    display_name: String,
    username_base: String,
    username_base_available: bool,
    username_suggestions: Vec<String>,
    selected_register_username: Option<String>,
    checking_usernames: bool,
    registering: bool,
    registration_access_key: Option<String>,
    fresh_recovery_key: Option<String>,
    saved_messages: Vec<SavedMessage>,
    saved_message_text: String,
    saved_messages_loaded: bool,
    profile_about: String,
    profile_display_name: String,
    profile_username: String,
    saving_profile: bool,
    group_creation_mode: Option<GroupCreationMode>,
    group_name: String,
    selected_group_members: HashSet<String>,
    group_post_policy: String,
    creating_group: bool,
    connection_page: ConnectionPage,
    login_mode: LoginMode,
    access_key: String,
    email: String,
    password: String,
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

        let environment_server = std::env::var("LARPTRIX_SERVER_URL").unwrap_or_default();
        let mut saved_servers = load_saved_servers();
        if !environment_server.trim().is_empty()
            && !saved_servers.iter().any(|saved| saved.eq_ignore_ascii_case(environment_server.trim()))
        {
            saved_servers.insert(0, environment_server.trim().to_string());
        }
        let server_url = if environment_server.trim().is_empty() {
            saved_servers.first().cloned().unwrap_or_default()
        } else {
            environment_server
        };
        let server_url = server_input_value(&server_url);

        Self {
            runtime,
            event_tx,
            core: ClientCore::new(),
            outgoing: None,
            crypto: None,
            decrypted_messages: HashMap::new(),
            decrypted_attachments: HashMap::new(),
            cached_attachment_paths: HashMap::new(),
            loading_attachment_ids: HashSet::new(),
            cached_avatar_paths: HashMap::new(),
            loading_avatar_ids: HashSet::new(),
            media_cache_dir: tempfile::Builder::new()
                .prefix("larptrix-media-")
                .tempdir()
                .ok()
                .map(Arc::new),
            image_viewer: None,
            media_player: None,
            media_session_id: None,
            media_title: String::new(),
            media_is_video: false,
            media_path: None,
            media_frame: None,
            media_position_ms: 0,
            media_duration_ms: 0,
            media_volume: 0.8,
            media_playing: false,
            media_finished: false,
            media_loading: false,
            media_error: None,
            pending_attachments: Vec::new(),
            pending_decryptions: HashMap::new(),
            decrypting_messages: HashSet::new(),
            server_url,
            saved_servers,
            api: None,
            friends: Vec::new(),
            incoming_friend_requests: Vec::new(),
            outgoing_friend_requests: Vec::new(),
            user_search: String::new(),
            user_search_results: Vec::new(),
            auth_mode: AuthMode::SignIn,
            display_name: String::new(),
            username_base: String::new(),
            username_base_available: false,
            username_suggestions: Vec::new(),
            selected_register_username: None,
            checking_usernames: false,
            registering: false,
            registration_access_key: None,
            fresh_recovery_key: None,
            saved_messages: Vec::new(),
            saved_message_text: String::new(),
            saved_messages_loaded: false,
            profile_about: String::new(),
            profile_display_name: String::new(),
            profile_username: String::new(),
            saving_profile: false,
            group_creation_mode: None,
            group_name: String::new(),
            selected_group_members: HashSet::new(),
            group_post_policy: "admins".into(),
            creating_group: false,
            connection_page: ConnectionPage::SignIn,
            login_mode: LoginMode::AccessKey,
            access_key: String::new(),
            email: String::new(),
            password: String::new(),
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
            AppEvent::Authenticated { user, outgoing, crypto, api, generated_recovery_key } => {
                self.fresh_recovery_key = generated_recovery_key;
                self.connecting = false;
                self.access_key.clear();
                self.email.clear();
                self.password.clear();
                self.recovery_key.clear();
                self.outgoing = Some(outgoing);
                self.crypto = Some(crypto.clone());
                self.api = Some(api.clone());
                self.preload_user_avatar(&user);
                self.profile_display_name = user.display_name.clone();
                self.profile_username = user.username.clone();
                let profile_user_id = user.user_id.clone();
                let saved_crypto = crypto.clone();
                self.core.apply_server_message(ServerMessage::Welcome {
                    user,
                    users: Vec::new(),
                });
                let events = self.event_tx.clone();
                self.runtime.spawn(async move {
                    match api.load_profile(&profile_user_id).await {
                        Ok(about) => {
                            let _ = events.send_blocking(AppEvent::ProfileSettingsLoaded { about });
                        }
                        Err(error) => {
                            let _ = events.send_blocking(AppEvent::Status(format!("Could not load profile settings: {error}")));
                        }
                    }
                });

                let events = self.event_tx.clone();
                self.runtime.spawn(async move {
                    let result = async {
                        let path = saved_crypto.saved_messages_path()?;
                        match tokio::fs::read(path).await {
                            Ok(encrypted) => {
                                let plaintext = saved_crypto.decrypt_local_data(&encrypted)?;
                                String::from_utf8(plaintext)
                                    .map_err(|error| format!("Saved messages are not valid UTF-8: {error}"))
                            }
                            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok("[]".to_string()),
                            Err(error) => Err(format!("Could not read saved messages: {error}")),
                        }
                    }.await;

                    match result {
                        Ok(data) => {
                            let _ = events.send_blocking(AppEvent::SavedMessagesLoaded { data });
                        }
                        Err(error) => {
                            let _ = events.send_blocking(AppEvent::SavedMessagesLoadFailed(error));
                        }
                    }
                });
                self.connection_status = "Signed in · synchronizing".into();
                self.active_section = Section::Messages;
            }
            AppEvent::UsernameSuggestions { base_username, base_available, suggestions } => {
                self.checking_usernames = false;
                self.registering = false;
                self.username_base = base_username;
                self.username_base_available = base_available;
                self.username_suggestions = suggestions;
                self.selected_register_username = None;
                self.connection_status = if base_available {
                    "Your username is available. Creating your account…".into()
                } else {
                    "That username is already taken. Choose one of these alternatives.".into()
                };
            }
            AppEvent::RegistrationFinished { user, access_key } => {
                self.registering = false;
                self.checking_usernames = false;
                self.auth_mode = AuthMode::SignIn;
                self.login_mode = LoginMode::AccessKey;
                self.access_key = access_key.clone();
                self.registration_access_key = Some(access_key);
                self.username_suggestions.clear();
                self.selected_register_username = None;
                self.connection_status = format!("Account created · your username is @{}", user.username);
            }
            AppEvent::RegistrationFailed(error) => {
                self.registering = false;
                self.checking_usernames = false;
                self.connection_status = format!("Registration failed: {error}");
            }
            AppEvent::FriendListsLoaded { friends, incoming, outgoing } => {
                for user in friends.iter().chain(incoming.iter()).chain(outgoing.iter()) {
                    self.preload_user_avatar(user);
                }
                let mut directory = Vec::new();
                for user in friends.iter().chain(incoming.iter()).chain(outgoing.iter()) {
                    if !directory.iter().any(|existing: &UserInfo| existing.user_id == user.user_id) {
                        directory.push(user.clone());
                    }
                }
                self.core.apply_server_message(ServerMessage::Directory { users: directory });
                let mut statuses = HashMap::new();
                for user in &friends {
                    statuses.insert(user.user_id.clone(), "accepted".to_string());
                }
                for user in &incoming {
                    statuses.insert(user.user_id.clone(), "pending_incoming".to_string());
                }
                for user in &outgoing {
                    statuses.insert(user.user_id.clone(), "pending_outgoing".to_string());
                }
                for user in &mut self.user_search_results {
                    if let Some(status) = statuses.get(&user.user_id) {
                        user.friend_status = status.clone();
                    }
                }
                self.friends = friends;
                self.incoming_friend_requests = incoming;
                self.outgoing_friend_requests = outgoing;
            }
            AppEvent::UserSearchResults { users } => {
                for user in &users {
                    self.preload_user_avatar(user);
                }
                self.core.apply_server_message(ServerMessage::Directory { users: users.clone() });
                self.user_search_results = users;
            }
            AppEvent::GroupCreated { group } => {
                let group_id = group.group_id.clone();
                let mut groups: Vec<_> = self.core.state().groups.values().cloned().collect();
                groups.retain(|existing| existing.group_id != group_id);
                groups.push(group);
                self.core.apply_server_message(ServerMessage::Groups { groups });
                self.group_creation_mode = None;
                self.group_name.clear();
                self.selected_group_members.clear();
                self.creating_group = false;
                self.active_section = Section::Messages;
                self.open_chat(group_id);
            }
            AppEvent::GroupCreationFailed(error) => {
                self.creating_group = false;
                self.connection_status = format!("Could not create group or channel: {error}");
            }
            AppEvent::ProfileSettingsLoaded { about } => {
                self.profile_about = about;
            }
            AppEvent::ProfileUpdated { user, about } => {
                self.saving_profile = false;
                self.profile_display_name = user.display_name.clone();
                self.profile_about = about;

                // /api/me returns a deliberately small public profile. Preserve
                // the richer current-user state already received over WebSocket.
                let mut merged_user = self.core.state().current_user.clone().unwrap_or(user);
                merged_user.display_name = self.profile_display_name.clone();
                merged_user.username = self.profile_username.clone();
                self.core.apply_server_message(ServerMessage::Welcome {
                    user: merged_user,
                    users: Vec::new(),
                });
                self.connection_status = "Profile updated".into();
            }
            AppEvent::ProfileUpdateFailed(error) => {
                self.saving_profile = false;
                self.connection_status = format!("Profile update failed: {error}");
            }
            AppEvent::MessagePolicyUpdated { user } => {
                // Preserve local device/E2E metadata that /api/me/settings does not return.
                let mut merged_user = self.core.state().current_user.clone().unwrap_or_else(|| user.clone());
                merged_user.message_policy = user.message_policy;
                self.core.apply_server_message(ServerMessage::Welcome {
                    user: merged_user,
                    users: Vec::new(),
                });
                self.connection_status = "Message privacy updated".into();
            }
            AppEvent::SavedMessagesLoaded { data } => {
                match serde_json::from_str::<Vec<SavedMessage>>(&data) {
                    Ok(messages) => {
                        self.saved_messages = messages;
                        self.saved_messages_loaded = true;
                    }
                    Err(error) => {
                        self.saved_messages_loaded = false;
                        self.connection_status = format!("Could not decode saved messages; existing file was left untouched: {error}");
                    }
                }
            }
            AppEvent::SavedMessagesLoadFailed(error) => {
                self.saved_messages_loaded = false;
                self.connection_status = format!("Could not unlock saved messages; existing file was left untouched: {error}");
            }
            AppEvent::SavedMessagesSaveFinished { error } => {
                if let Some(error) = error {
                    self.connection_status = format!("Could not save local notes: {error}");
                } else {
                    self.connection_status = "Saved messages updated · encrypted on this device".into();
                }
            }
            AppEvent::AvatarCached { user_id, path, error } => {
                self.loading_avatar_ids.remove(&user_id);
                if error.is_none() && !path.as_os_str().is_empty() {
                    self.cached_avatar_paths.insert(user_id, path);
                }
            }
            AppEvent::Server(message) => {
                let avatar_users: Vec<UserInfo> = match &message {
                    ServerMessage::Welcome { user, users } => {
                        std::iter::once(user.clone()).chain(users.iter().cloned()).collect()
                    }
                    ServerMessage::Directory { users } => users.clone(),
                    ServerMessage::Chat { peer, .. } => vec![peer.clone()],
                    _ => Vec::new(),
                };
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
                for user in &avatar_users {
                    self.preload_user_avatar(user);
                }
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
                    for attachment in attachments.iter().filter(|item| item.mime.starts_with("image/")) {
                        self.preload_inline_image(attachment.clone());
                    }
                    self.decrypted_attachments.insert(key, attachments);
                }
            }
            AppEvent::ImageCached { attachment_id, path, error } => {
                self.loading_attachment_ids.remove(&attachment_id);
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
            AppEvent::MediaOpened { session_id, title, is_video, path, player } => {
                if self.media_session_id.as_deref() != Some(session_id.as_str()) {
                    player.stop();
                    let _ = std::fs::remove_file(path);
                } else {
                    self.media_loading = false;
                    self.media_title = title;
                    self.media_is_video = is_video;
                    self.media_path = Some(path);
                    self.media_frame = None;
                    self.media_position_ms = 0;
                    self.media_duration_ms = player.duration_ms();
                    self.media_playing = player.is_playing();
                    self.media_finished = false;
                    self.media_error = None;
                    player.set_volume(self.media_volume);
                    self.media_player = Some(player.clone());
                    self.start_media_monitor(session_id, player);
                }
            }
            AppEvent::MediaTick { session_id, position_ms, duration_ms, finished, error } => {
                if self.media_session_id.as_deref() == Some(session_id.as_str()) {
                    self.media_position_ms = position_ms;
                    self.media_duration_ms = duration_ms.max(self.media_duration_ms);
                    self.media_finished = finished;
                    if finished {
                        self.media_playing = false;
                    }
                    if let Some(player) = self.media_player.as_ref() {
                        self.media_frame = player.latest_video_frame();
                    }
                    if let Some(error) = error {
                        self.media_error = Some(error);
                        self.media_playing = false;
                    }
                }
            }
            AppEvent::MediaFailed { session_id, error } => {
                if self.media_session_id.as_deref() == Some(session_id.as_str()) {
                    self.media_loading = false;
                    self.media_error = Some(error);
                    self.media_playing = false;
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

    fn choose_server(&mut self, cx: &mut Context<Self>) {
        let raw = self.server_url.trim();
        if raw.is_empty() {
            self.connection_status = "Enter a server address first".into();
            cx.notify();
            return;
        }
        let candidate = if raw.contains("://") {
            raw.to_string()
        } else {
            format!("https://{raw}")
        };
        let normalized = match network::normalize_server_url(&candidate) {
            Ok(url) => url.to_string(),
            Err(error) => {
                self.connection_status = error;
                cx.notify();
                return;
            }
        };

        self.server_url = normalized.clone();
        if !self.saved_servers.iter().any(|saved| saved.eq_ignore_ascii_case(&normalized)) {
            self.saved_servers.push(normalized);
            if self.saved_servers.len() > 20 {
                self.saved_servers.remove(0);
            }
            save_saved_servers(&self.saved_servers);
        }
        self.connection_page = ConnectionPage::SignIn;
        self.connection_status = "Server selected. Sign in to continue.".into();
        self.active_input = Some(match self.login_mode {
            LoginMode::AccessKey => LoginField::AccessKey,
            LoginMode::EmailPassword => LoginField::Email,
        });
        cx.notify();
    }

    fn start_connection(&mut self, cx: &mut Context<Self>) {
        if self.connecting || self.core.state().current_user.is_some() {
            return;
        }

        let candidate = self.server_url.trim().to_string();
        let server_url = match network::normalize_server_url(&candidate) {
            Ok(url) => url.to_string(),
            Err(error) => {
                self.connection_status = error;
                cx.notify();
                return;
            }
        };
        let recovery_key = self.recovery_key.trim().to_string();
        self.server_url = server_input_value(&server_url);
        if !self.saved_servers.iter().any(|saved| saved.eq_ignore_ascii_case(&server_url)) {
            self.saved_servers.push(server_url.clone());
            if self.saved_servers.len() > 20 {
                self.saved_servers.remove(0);
            }
            save_saved_servers(&self.saved_servers);
        }

        let credentials = match self.login_mode {
            LoginMode::AccessKey => {
                let entered_key = self.access_key.trim().to_string();
                let access_key: String = entered_key
                    .chars()
                    .filter(|ch| ch.is_ascii_hexdigit())
                    .collect();
                let key_format_is_valid = entered_key
                    .chars()
                    .all(|ch| ch.is_ascii_hexdigit() || ch == '-' || ch.is_ascii_whitespace());
                if !key_format_is_valid || access_key.len() != 64 {
                    self.connection_status =
                        "Enter the 64-hex access key (with or without grouping dashes)".into();
                    cx.notify();
                    return;
                }
                network::LoginCredentials::AccessKey(access_key)
            }
            LoginMode::EmailPassword => {
                let email = self.email.trim().to_string();
                if email.is_empty() || self.password.is_empty() {
                    self.connection_status = "Enter your account email and password".into();
                    cx.notify();
                    return;
                }
                network::LoginCredentials::EmailPassword {
                    email,
                    password: self.password.clone(),
                }
            }
        };

        self.connecting = true;
        self.connection_status = "Starting secure session…".into();
        cx.notify();

        let events: Sender<AppEvent> = Sender::clone(&self.event_tx);
        self.runtime.spawn(async move {
            if let Err(error) = network::connect(server_url, credentials, recovery_key, events.clone()).await {
                let _ = events.send_blocking(AppEvent::Error(error));
            }
        });
    }

    fn start_registration(&mut self, cx: &mut Context<Self>) {
        if self.registering || self.connecting {
            return;
        }
        let display_name = self.display_name.trim().to_string();
        if display_name.is_empty() {
            self.connection_status = "Enter your display name first".into();
            cx.notify();
            return;
        }
        let server_address = self.server_url.trim().to_string();
        if server_address.is_empty() {
            self.connection_status = "Enter your server address first".into();
            cx.notify();
            return;
        }
        if let Err(error) = network::normalize_server_url(&server_address) {
            self.connection_status = error;
            cx.notify();
            return;
        }

        let selected_username = self.selected_register_username.clone();
        let display_for_request = display_name.clone();
        self.registering = true;
        self.connection_status = "Checking username availability…".into();
        cx.notify();
        let events = self.event_tx.clone();
        self.runtime.spawn(async move {
            let api = match NativeApi::anonymous(&server_address) {
                Ok(api) => api,
                Err(error) => {
                    let _ = events.send_blocking(AppEvent::RegistrationFailed(error));
                    return;
                }
            };

            if let Some(username) = selected_username {
                match api.register(&display_for_request, &username).await {
                    Ok(response) => {
                        let _ = events.send_blocking(AppEvent::RegistrationFinished {
                            user: response.user,
                            access_key: response.access_key,
                        });
                    }
                    Err(error) => {
                        if error.to_ascii_lowercase().contains("username") {
                            match api.suggest_usernames(&display_for_request).await {
                                Ok(suggestions) => {
                                    let _ = events.send_blocking(AppEvent::UsernameSuggestions {
                                        base_username: suggestions.base_username,
                                        base_available: suggestions.base_available,
                                        suggestions: suggestions.suggestions,
                                    });
                                }
                                Err(suggest_error) => {
                                    let _ = events.send_blocking(AppEvent::RegistrationFailed(suggest_error));
                                }
                            }
                        } else {
                            let _ = events.send_blocking(AppEvent::RegistrationFailed(error));
                        }
                    }
                }
                return;
            }

            match api.suggest_usernames(&display_for_request).await {
                Ok(suggestions) if suggestions.base_available => {
                    match api.register(&display_for_request, &suggestions.base_username).await {
                        Ok(response) => {
                            let _ = events.send_blocking(AppEvent::RegistrationFinished {
                                user: response.user,
                                access_key: response.access_key,
                            });
                        }
                        Err(error) if error.to_ascii_lowercase().contains("username") => {
                            match api.suggest_usernames(&display_for_request).await {
                                Ok(suggestions) => {
                                    let _ = events.send_blocking(AppEvent::UsernameSuggestions {
                                        base_username: suggestions.base_username,
                                        base_available: suggestions.base_available,
                                        suggestions: suggestions.suggestions,
                                    });
                                }
                                Err(suggest_error) => {
                                    let _ = events.send_blocking(AppEvent::RegistrationFailed(suggest_error));
                                }
                            }
                        }
                        Err(error) => {
                            let _ = events.send_blocking(AppEvent::RegistrationFailed(error));
                        }
                    }
                }
                Ok(suggestions) => {
                    let _ = events.send_blocking(AppEvent::UsernameSuggestions {
                        base_username: suggestions.base_username,
                        base_available: suggestions.base_available,
                        suggestions: suggestions.suggestions,
                    });
                }
                Err(error) => {
                    let _ = events.send_blocking(AppEvent::RegistrationFailed(error));
                }
            }
        });
    }

    fn select_register_username(&mut self, username: String, cx: &mut Context<Self>) {
        if self.username_suggestions.iter().any(|candidate| candidate == &username) {
            self.selected_register_username = Some(username);
            cx.notify();
        }
    }

    fn load_friend_lists(&mut self, cx: &mut Context<Self>) {
        let Some(api) = self.api.clone() else {
            self.connection_status = "Friends are unavailable until the server connection is ready".into();
            cx.notify();
            return;
        };
        let events = self.event_tx.clone();
        self.runtime.spawn(async move {
            match api.load_friends().await {
                Ok(data) => {
                    let _ = events.send_blocking(AppEvent::FriendListsLoaded {
                        friends: data.friends,
                        incoming: data.incoming,
                        outgoing: data.outgoing,
                    });
                }
                Err(error) => {
                    let _ = events.send_blocking(AppEvent::Status(format!("Could not load friends: {error}")));
                }
            }
        });
    }

    fn search_users(&mut self, cx: &mut Context<Self>) {
        let query = self.user_search.trim().to_string();
        if query.is_empty() {
            self.connection_status = "Enter a username or display name to search".into();
            self.user_search_results.clear();
            cx.notify();
            return;
        }
        let Some(api) = self.api.clone() else {
            self.connection_status = "User search is unavailable until connected".into();
            cx.notify();
            return;
        };

        self.connection_status = format!("Searching for {query}…");
        cx.notify();
        let events = self.event_tx.clone();
        self.runtime.spawn(async move {
            match api.search_users(&query).await {
                Ok(users) => {
                    let _ = events.send_blocking(AppEvent::UserSearchResults { users });
                    let _ = events.send_blocking(AppEvent::Status("User search completed".into()));
                }
                Err(error) => {
                    let _ = events.send_blocking(AppEvent::Status(format!("User search failed: {error}")));
                }
            }
        });
    }

    fn send_friend_request(&mut self, user_id: String, cx: &mut Context<Self>) {
        let Some(api) = self.api.clone() else {
            self.connection_status = "Friend requests are unavailable until connected".into();
            cx.notify();
            return;
        };
        self.connection_status = "Sending friend request…".into();
        cx.notify();
        let events = self.event_tx.clone();
        self.runtime.spawn(async move {
            match api.send_friend_request(&user_id).await {
                Ok(()) => {
                    let _ = events.send_blocking(AppEvent::Status("Friend request sent".into()));
                    if let Ok(data) = api.load_friends().await {
                        let _ = events.send_blocking(AppEvent::FriendListsLoaded {
                            friends: data.friends,
                            incoming: data.incoming,
                            outgoing: data.outgoing,
                        });
                    }
                }
                Err(error) => {
                    let _ = events.send_blocking(AppEvent::Status(format!("Could not send friend request: {error}")));
                }
            }
        });
    }

    fn accept_friend_request(&mut self, user_id: String, cx: &mut Context<Self>) {
        let Some(api) = self.api.clone() else {
            self.connection_status = "Friend requests are unavailable until connected".into();
            cx.notify();
            return;
        };
        self.connection_status = "Accepting friend request…".into();
        cx.notify();
        let events = self.event_tx.clone();
        self.runtime.spawn(async move {
            match api.accept_friend_request(&user_id).await {
                Ok(()) => {
                    let _ = events.send_blocking(AppEvent::Status("Friend request accepted".into()));
                    if let Ok(data) = api.load_friends().await {
                        let _ = events.send_blocking(AppEvent::FriendListsLoaded {
                            friends: data.friends,
                            incoming: data.incoming,
                            outgoing: data.outgoing,
                        });
                    }
                }
                Err(error) => {
                    let _ = events.send_blocking(AppEvent::Status(format!("Could not accept friend request: {error}")));
                }
            }
        });
    }

    fn persist_saved_messages(&mut self, cx: &mut Context<Self>) {
        let Some(crypto) = self.crypto.clone() else {
            self.connection_status = "E2E storage is locked".into();
            cx.notify();
            return;
        };
        let data = match serde_json::to_vec(&self.saved_messages) {
            Ok(data) => data,
            Err(error) => {
                self.connection_status = format!("Could not encode saved messages: {error}");
                cx.notify();
                return;
            }
        };
        let encrypted = match crypto.encrypt_local_data(&data) {
            Ok(encrypted) => encrypted,
            Err(error) => {
                self.connection_status = error;
                cx.notify();
                return;
            }
        };
        let path = match crypto.saved_messages_path() {
            Ok(path) => path,
            Err(error) => {
                self.connection_status = error;
                cx.notify();
                return;
            }
        };
        let events = self.event_tx.clone();
        self.runtime.spawn(async move {
            let result = tokio::fs::write(&path, encrypted).await
                .map_err(|error| format!("Could not write saved-message store: {error}"));
            let _ = events.send_blocking(AppEvent::SavedMessagesSaveFinished {
                error: result.err(),
            });
        });
        cx.notify();
    }

    fn save_saved_message(&mut self, cx: &mut Context<Self>) {
        let body = self.saved_message_text.trim().to_string();
        if body.is_empty() || !self.saved_messages_loaded {
            return;
        }
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs() as i64)
            .unwrap_or_default();
        self.saved_messages.push(SavedMessage {
            id: uuid::Uuid::new_v4().to_string(),
            body,
            created_at,
        });
        self.saved_message_text.clear();
        self.persist_saved_messages(cx);
    }

    fn delete_saved_message(&mut self, id: &str, cx: &mut Context<Self>) {
        self.saved_messages.retain(|message| message.id != id);
        self.persist_saved_messages(cx);
    }

    fn save_profile(&mut self, cx: &mut Context<Self>) {
        if self.saving_profile {
            return;
        }
        let Some(api) = self.api.clone() else {
            self.connection_status = "Server API is not ready".into();
            cx.notify();
            return;
        };
        let display_name = self.profile_display_name.trim().to_string();
        let username = self.profile_username.trim().to_string();
        let about = self.profile_about.trim().to_string();
        let tags = self.core.state().current_user.as_ref()
            .map(|user| user.tags.clone())
            .unwrap_or_default();
        if display_name.is_empty() {
            self.connection_status = "Display name cannot be empty".into();
            cx.notify();
            return;
        }

        self.saving_profile = true;
        self.connection_status = "Saving profile…".into();
        cx.notify();
        let events = self.event_tx.clone();
        self.runtime.spawn(async move {
            match api.update_profile(&display_name, &username, &about, &tags).await {
                Ok(user) => {
                    let _ = events.send_blocking(AppEvent::ProfileUpdated { user, about });
                }
                Err(error) => {
                    let _ = events.send_blocking(AppEvent::ProfileUpdateFailed(error));
                }
            }
        });
    }

    fn toggle_message_policy(&mut self, cx: &mut Context<Self>) {
        let current = self.core.state().current_user.as_ref()
            .map(|user| user.message_policy.as_str())
            .unwrap_or("everyone");
        let next = if current == "friends" { "everyone" } else { "friends" };
        let Some(api) = self.api.clone() else {
            self.connection_status = "Server API is not ready".into();
            cx.notify();
            return;
        };
        self.connection_status = "Updating message privacy…".into();
        cx.notify();
        let events = self.event_tx.clone();
        let policy = next.to_string();
        self.runtime.spawn(async move {
            match api.update_message_policy(&policy).await {
                Ok(user) => {
                    let _ = events.send_blocking(AppEvent::MessagePolicyUpdated { user });
                }
                Err(error) => {
                    let _ = events.send_blocking(AppEvent::Status(format!("Message privacy update failed: {error}")));
                }
            }
        });
    }

    fn create_group_or_channel(&mut self, cx: &mut Context<Self>) {
        if self.creating_group {
            return;
        }
        let name = self.group_name.trim().to_string();
        if name.is_empty() {
            self.connection_status = "Enter a name first".into();
            cx.notify();
            return;
        }
        let Some(mode) = self.group_creation_mode else {
            return;
        };
        let mut member_ids: Vec<String> = self.selected_group_members.iter().cloned().collect();
        member_ids.sort();
        if mode == GroupCreationMode::Group && member_ids.is_empty() {
            self.connection_status = "Choose at least one friend for a group".into();
            cx.notify();
            return;
        }
        let Some(api) = self.api.clone() else {
            self.connection_status = "Server API is not ready".into();
            cx.notify();
            return;
        };

        let post_policy = self.group_post_policy.clone();
        self.creating_group = true;
        self.connection_status = match mode {
            GroupCreationMode::Group => "Creating encrypted group…".into(),
            GroupCreationMode::Channel => "Creating encrypted channel…".into(),
        };
        cx.notify();

        let events = self.event_tx.clone();
        self.runtime.spawn(async move {
            let result = match mode {
                GroupCreationMode::Group => api.create_group(&name, &member_ids).await,
                GroupCreationMode::Channel => api.create_channel(&name, &member_ids, &post_policy).await,
            };
            match result {
                Ok(group) => {
                    let _ = events.send_blocking(AppEvent::GroupCreated { group });
                }
                Err(error) => {
                    let _ = events.send_blocking(AppEvent::GroupCreationFailed(error));
                }
            }
        });
    }

    fn toggle_group_member(&mut self, user_id: String, cx: &mut Context<Self>) {
        if !self.selected_group_members.insert(user_id.clone()) {
            self.selected_group_members.remove(&user_id);
        }
        cx.notify();
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
                LoginField::DisplayName => { self.display_name.pop(); }
                LoginField::AccessKey => { self.access_key.pop(); }
                LoginField::Email => { self.email.pop(); }
                LoginField::Password => { self.password.pop(); }
                LoginField::RecoveryKey => { self.recovery_key.pop(); }
                LoginField::Composer => { self.composer_text.pop(); }
                LoginField::PeopleSearch => { self.user_search.pop(); }
                LoginField::GroupName => { self.group_name.pop(); }
                LoginField::ProfileDisplayName => { self.profile_display_name.pop(); }
                LoginField::ProfileUsername => { self.profile_username.pop(); }
                LoginField::ProfileAbout => { self.profile_about.pop(); }
                LoginField::SavedComposer => { self.saved_message_text.pop(); }
            }
            cx.notify();
            return;
        }

        if event.keystroke.key == "enter" {
            match field {
                LoginField::Composer => self.send_message(cx),
                LoginField::Server => {
                    if self.auth_mode == AuthMode::Register {
                        self.start_registration(cx);
                    } else {
                        self.start_connection(cx);
                    }
                }
                LoginField::DisplayName => self.start_registration(cx),
                LoginField::PeopleSearch => self.search_users(cx),
                LoginField::GroupName => self.create_group_or_channel(cx),
                LoginField::ProfileDisplayName | LoginField::ProfileUsername | LoginField::ProfileAbout => self.save_profile(cx),
                LoginField::SavedComposer => self.save_saved_message(cx),
                _ => self.start_connection(cx),
            }
            return;
        }

        let is_paste = event.keystroke.key == "v"
            && (event.keystroke.modifiers.control || event.keystroke.modifiers.platform);
        if is_paste {
            if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                let current_len = match field {
                    LoginField::Server => self.server_url.len(),
                    LoginField::DisplayName => self.display_name.len(),
                    LoginField::AccessKey => self.access_key.len(),
                    LoginField::Email => self.email.len(),
                    LoginField::Password => self.password.len(),
                    LoginField::RecoveryKey => self.recovery_key.len(),
                    LoginField::Composer => self.composer_text.len(),
                    LoginField::PeopleSearch => self.user_search.len(),
                    LoginField::GroupName => self.group_name.len(),
                    LoginField::ProfileDisplayName => self.profile_display_name.len(),
                    LoginField::ProfileUsername => self.profile_username.len(),
                    LoginField::ProfileAbout => self.profile_about.len(),
                    LoginField::SavedComposer => self.saved_message_text.len(),
                };
                let max_len = match field {
                    LoginField::Server => 256usize,
                    LoginField::DisplayName => 32usize,
                    LoginField::AccessKey => 80usize,
                    LoginField::Email => 320usize,
                    LoginField::Password => 256usize,
                    LoginField::RecoveryKey => 128usize,
                    LoginField::Composer => 12_000usize,
                    LoginField::PeopleSearch => 64usize,
                    LoginField::GroupName => 32usize,
                    LoginField::ProfileDisplayName => 32usize,
                    LoginField::ProfileUsername => 24usize,
                    LoginField::ProfileAbout => 160usize,
                    LoginField::SavedComposer => 12_000usize,
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
                    LoginField::DisplayName => self.display_name.push_str(&pasted),
                    LoginField::AccessKey => self.access_key.push_str(&pasted),
                    LoginField::Email => self.email.push_str(&pasted),
                    LoginField::Password => self.password.push_str(&pasted),
                    LoginField::RecoveryKey => self.recovery_key.push_str(&pasted),
                    LoginField::Composer => self.composer_text.push_str(&pasted),
                    LoginField::PeopleSearch => self.user_search.push_str(&pasted),
                    LoginField::GroupName => self.group_name.push_str(&pasted),
                    LoginField::ProfileDisplayName => self.profile_display_name.push_str(&pasted),
                    LoginField::ProfileUsername => self.profile_username.push_str(&pasted),
                    LoginField::ProfileAbout => self.profile_about.push_str(&pasted),
                    LoginField::SavedComposer => self.saved_message_text.push_str(&pasted),
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
                LoginField::DisplayName => (self.display_name.len(), 32usize),
                LoginField::AccessKey => (self.access_key.len(), 80usize),
                LoginField::Email => (self.email.len(), 320usize),
                LoginField::Password => (self.password.len(), 256usize),
                LoginField::RecoveryKey => (self.recovery_key.len(), 128usize),
                LoginField::Composer => (self.composer_text.len(), 12_000usize),
                LoginField::PeopleSearch => (self.user_search.len(), 64usize),
                LoginField::GroupName => (self.group_name.len(), 32usize),
                LoginField::ProfileDisplayName => (self.profile_display_name.len(), 32usize),
                LoginField::ProfileUsername => (self.profile_username.len(), 24usize),
                LoginField::ProfileAbout => (self.profile_about.len(), 160usize),
                LoginField::SavedComposer => (self.saved_message_text.len(), 12_000usize),
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
                LoginField::DisplayName => self.display_name.push_str(text),
                LoginField::AccessKey => self.access_key.push_str(text),
                LoginField::Email => self.email.push_str(text),
                LoginField::Password => self.password.push_str(text),
                LoginField::RecoveryKey => self.recovery_key.push_str(text),
                LoginField::Composer => self.composer_text.push_str(text),
                LoginField::PeopleSearch => self.user_search.push_str(text),
                LoginField::GroupName => self.group_name.push_str(text),
                LoginField::ProfileDisplayName => self.profile_display_name.push_str(text),
                LoginField::ProfileUsername => self.profile_username.push_str(text),
                LoginField::ProfileAbout => self.profile_about.push_str(text),
                LoginField::SavedComposer => self.saved_message_text.push_str(text),
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
        let display_value = if matches!(field, LoginField::AccessKey | LoginField::Password | LoginField::RecoveryKey) && !value.is_empty() {
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
                        LoginField::DisplayName => "display-name-input",
                        LoginField::AccessKey => "access-key-input",
                        LoginField::Email => "email-input",
                        LoginField::Password => "password-input",
                        LoginField::RecoveryKey => "recovery-key-input",
                        LoginField::Composer => "message-composer",
                        LoginField::PeopleSearch => "people-search-input",
                        LoginField::GroupName => "group-name-input",
                        LoginField::ProfileDisplayName => "profile-display-name-input",
                        LoginField::ProfileUsername => "profile-username-input",
                        LoginField::ProfileAbout => "profile-about-input",
                        LoginField::SavedComposer => "saved-message-composer",
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
            self.open_media_inline(metadata, cx);
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
        self.close_media_player();
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

    fn preload_user_avatar(&mut self, user: &UserInfo) {
        let Some(asset_url) = user.avatar_url.as_deref().filter(|url| !url.trim().is_empty()) else {
            return;
        };
        if self.cached_avatar_paths.contains_key(&user.user_id)
            || !self.loading_avatar_ids.insert(user.user_id.clone())
        {
            return;
        }

        let Some(api) = self.api.clone() else {
            self.loading_avatar_ids.remove(&user.user_id);
            return;
        };
        let Some(cache_dir) = self.media_cache_dir.clone() else {
            self.loading_avatar_ids.remove(&user.user_id);
            return;
        };

        let user_id = user.user_id.clone();
        let asset_url = asset_url.to_string();
        let events = self.event_tx.clone();
        self.runtime.spawn(async move {
            let result = async {
                let (bytes, mime) = api.download_asset(&asset_url).await?;
                let extension = media_extension(&mime);
                let path = cache_dir.path().join(format!("avatar-{}.{}", uuid::Uuid::new_v4(), extension));
                tokio::fs::write(&path, bytes)
                    .await
                    .map_err(|error| format!("Could not cache profile image: {error}"))?;
                Ok::<PathBuf, String>(path)
            }.await;

            let event = match result {
                Ok(path) => AppEvent::AvatarCached {
                    user_id,
                    path,
                    error: None,
                },
                Err(error) => AppEvent::AvatarCached {
                    user_id,
                    path: PathBuf::new(),
                    error: Some(error),
                },
            };
            let _ = events.send_blocking(event);
        });
    }

    fn preload_inline_image(&mut self, metadata: EncryptedAttachmentMetadata) {
        if self.cached_attachment_paths.contains_key(&metadata.id)
            || !self.loading_attachment_ids.insert(metadata.id.clone())
        {
            return;
        }

        let Some(crypto) = self.crypto.clone() else {
            self.loading_attachment_ids.remove(&metadata.id);
            return;
        };
        let Some(cache_dir) = self.media_cache_dir.clone() else {
            self.loading_attachment_ids.remove(&metadata.id);
            return;
        };

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

    fn load_viewer_image(&mut self, _cx: &mut Context<Self>) {
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

    fn open_media_inline(
        &mut self,
        metadata: EncryptedAttachmentMetadata,
        cx: &mut Context<Self>,
    ) {
        self.close_media_player();
        self.image_viewer = None;
        self.media_title = metadata.name.clone();
        self.media_is_video = metadata.mime.starts_with("video/");
        let Some(crypto) = self.crypto.clone() else {
            self.media_error = Some("E2E is locked; media cannot be decrypted".into());
            cx.notify();
            return;
        };
        let Some(cache_dir) = self.media_cache_dir.clone() else {
            self.media_error = Some("Could not create a private temporary media directory".into());
            cx.notify();
            return;
        };

        let is_video = metadata.mime.starts_with("video/");
        let title = metadata.name.clone();
        let session_id = uuid::Uuid::new_v4().to_string();
        self.media_session_id = Some(session_id.clone());
        self.media_title = title.clone();
        self.media_is_video = is_video;
        self.media_loading = true;
        self.media_error = None;
        self.media_position_ms = 0;
        self.media_duration_ms = 0;
        self.media_finished = false;
        self.media_frame = None;
        self.connection_status = format!("Decrypting {}…", metadata.name);
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

                let player_path = path.clone();
                let player_result = tokio::task::spawn_blocking(move || {
                    MediaPlayback::open(&player_path, is_video)
                })
                .await
                .map_err(|error| format!("Media decoder task failed: {error}"))
                .and_then(|result| result);

                match player_result {
                    Ok(player) => Ok::<(PathBuf, Arc<MediaPlayback>), String>((path, player)),
                    Err(error) => {
                        let _ = tokio::fs::remove_file(&path).await;
                        Err(error)
                    }
                }
            }
            .await;

            match result {
                Ok((path, player)) => {
                    let _ = events.send_blocking(AppEvent::MediaOpened {
                        session_id,
                        title,
                        is_video,
                        path,
                        player,
                    });
                }
                Err(error) => {
                    let _ = events.send_blocking(AppEvent::MediaFailed {
                        session_id,
                        error,
                    });
                }
            }
        });
    }

    fn start_media_monitor(&self, session_id: String, player: Arc<MediaPlayback>) {
        let events = self.event_tx.clone();
        self.runtime.spawn(async move {
            while player.is_active() {
                let (finished, bus_error) = match player.poll_bus() {
                    Ok(finished) => (finished, None),
                    Err(error) => (false, Some(error)),
                };
                let error = bus_error.or_else(|| player.poll_video_frame().err());
                let _ = events.send_blocking(AppEvent::MediaTick {
                    session_id: session_id.clone(),
                    position_ms: player.position_ms(),
                    duration_ms: player.duration_ms(),
                    finished,
                    error: error.clone(),
                });

                if finished || error.is_some() {
                    if error.is_some() {
                        player.stop();
                    }
                    break;
                }

                let delay = if player.is_playing() {
                    if player.latest_video_frame().is_some() {
                        Duration::from_millis(33)
                    } else {
                        Duration::from_millis(100)
                    }
                } else {
                    Duration::from_millis(250)
                };
                tokio::time::sleep(delay).await;
            }
        });
    }

    fn close_media_player(&mut self) {
        self.media_session_id = None;
        self.media_loading = false;
        if let Some(player) = self.media_player.take() {
            player.stop();
        }
        if let Some(path) = self.media_path.take() {
            let _ = std::fs::remove_file(path);
        }
        self.media_frame = None;
        self.media_position_ms = 0;
        self.media_duration_ms = 0;
        self.media_playing = false;
        self.media_finished = false;
        self.media_error = None;
    }

    fn toggle_media_playback(&mut self, cx: &mut Context<Self>) {
        let Some(player) = self.media_player.clone() else {
            return;
        };
        let was_finished = self.media_finished;
        let next = !self.media_playing;
        match player.set_playing(next) {
            Ok(()) => {
                self.media_playing = next;
                self.media_finished = false;
                if next && was_finished {
                    if let Some(session_id) = self.media_session_id.clone() {
                        self.start_media_monitor(session_id, player);
                    }
                }
            }
            Err(error) => self.media_error = Some(error),
        }
        cx.notify();
    }

    fn seek_media(&mut self, target_ms: u64, cx: &mut Context<Self>) {
        let Some(player) = self.media_player.clone() else {
            return;
        };
        let was_finished = self.media_finished;
        let target = if self.media_duration_ms > 0 {
            target_ms.min(self.media_duration_ms)
        } else {
            target_ms
        };
        match player.seek_to(target) {
            Ok(()) => {
                self.media_position_ms = target;
                self.media_finished = false;
                if was_finished {
                    if let Err(error) = player.set_playing(false) {
                        self.media_error = Some(error);
                    } else {
                        self.media_playing = false;
                        if let Some(session_id) = self.media_session_id.clone() {
                            self.start_media_monitor(session_id, player);
                        }
                    }
                }
            }
            Err(error) => self.media_error = Some(error),
        }
        cx.notify();
    }

    fn seek_media_by(&mut self, delta_ms: i64, cx: &mut Context<Self>) {
        let target = (self.media_position_ms as i128 + i128::from(delta_ms))
            .clamp(0, self.media_duration_ms as i128) as u64;
        self.seek_media(target, cx);
    }

    fn change_media_volume(&mut self, delta: f32, cx: &mut Context<Self>) {
        self.media_volume = (self.media_volume + delta).clamp(0.0, 1.0);
        if let Some(player) = self.media_player.as_ref() {
            player.set_volume(self.media_volume);
        }
        cx.notify();
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
                    "rail-saved",
                    "★",
                    self.active_section == Section::Saved,
                    cx.listener(|this, _, _, cx| {
                        this.active_section = Section::Saved;
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
                        this.load_friend_lists(cx);
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
            Section::Saved => self.saved_messages_sidebar().into_any_element(),
            Section::Friends => self.friends_list(cx).into_any_element(),
            Section::Groups => self.groups_list(cx).into_any_element(),
            Section::Settings => self.settings_list(cx).into_any_element(),
        };

        let (account_name, username, account_user_id) = self
            .core
            .state()
            .current_user
            .as_ref()
            .map(|user| (user.display_name.clone(), user.username.clone(), user.user_id.clone()))
            .unwrap_or_else(|| ("Account".into(), String::new(), String::new()));
        let account_avatar = self.cached_avatar_paths.get(&account_user_id).cloned();

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
                    .child(avatar_view(
                        account_avatar,
                        account_name.chars().next().unwrap_or('U').to_string(),
                        36.0,
                    ))
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

    fn saved_messages_sidebar(&self) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .child(section_heading("PRIVATE LOCAL STORAGE"))
            .child(settings_row(
                "Saved Messages",
                format!("{} saved note(s) · encrypted on this device", self.saved_messages.len()),
            ))
            .child(empty_hint("These notes are stored locally and are not sent to your Larptrix server."))
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
                let avatar = self.cached_avatar_paths.get(&peer_id).cloned();
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
                (peer_id, name, preview, online, avatar)
            })
            .collect();
        let no_peers = rows.is_empty();

        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_1()
            .px_2()
            .children(rows.into_iter().map(|(peer_id, name, preview, online, avatar)| {
                let selected = self.selected_peer.as_deref() == Some(peer_id.as_str());
                let click_id = peer_id.clone();
                chat_entry(
                    peer_id,
                    name,
                    preview,
                    if online { "ONLINE" } else { "OFFLINE" }.into(),
                    avatar,
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

    fn friends_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let friends = self.friends.clone();
        let incoming = self.incoming_friend_requests.clone();
        let outgoing = self.outgoing_friend_requests.clone();
        let results = self.user_search_results.clone();
        let query = self.user_search.trim().to_string();

        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .child(section_heading("FIND PEOPLE"))
            .child(self.login_field(
                LoginField::PeopleSearch,
                "USERNAME OR DISPLAY NAME",
                "@username",
                &self.user_search,
                cx,
            ))
            .child(
                div()
                    .id("search-users-button")
                    .w_full()
                    .p_2()
                    .rounded_md()
                    .bg(rgb(ACCENT))
                    .text_color(rgb(RAIL))
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| this.search_users(cx)))
                    .child("Search users"),
            )
            .children(results.iter().map(|user| {
                let user_id = user.user_id.clone();
                let name = user.display_name.clone();
                let username = user.username.clone();
                let online = user.online;
                let status = user.friend_status.clone();
                let is_none = status == "none";
                let is_incoming = status == "pending_incoming";
                let action_label = match status.as_str() {
                    "accepted" => "Friends",
                    "pending_outgoing" => "Request sent",
                    "pending_incoming" => "Accept",
                    _ => "Add friend",
                };
                div()
                    .id(format!("search-result-{user_id}"))
                    .p_2()
                    .rounded_md()
                    .bg(rgb(SURFACE))
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(avatar_view(
                        self.cached_avatar_paths.get(&user_id).cloned(),
                        name.chars().next().unwrap_or('?').to_string(),
                        32.0,
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(div().text_sm().text_color(rgb(TEXT)).child(name))
                            .child(div().text_xs().text_color(rgb(MUTED)).child(
                                if username.is_empty() {
                                    if online { "Online".into() } else { "Offline".into() }
                                } else {
                                    format!("@{username} · {}", if online { "Online" } else { "Offline" })
                                },
                            )),
                    )
                    .child(div().flex_1())
                    .when(is_none, |element| {
                        let click_id = user_id.clone();
                        element.child(
                            div()
                                .id(format!("add-friend-{user_id}"))
                                .px_2()
                                .py_2()
                                .rounded_md()
                                .bg(rgb(SURFACE_HOVER))
                                .text_xs()
                                .text_color(rgb(ACCENT))
                                .cursor_pointer()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.send_friend_request(click_id.clone(), cx);
                                }))
                                .child(action_label),
                        )
                    })
                    .when(is_incoming, |element| {
                        let click_id = user_id.clone();
                        element.child(
                            div()
                                .id(format!("accept-search-friend-{user_id}"))
                                .px_2()
                                .py_2()
                                .rounded_md()
                                .bg(rgb(ACCENT))
                                .text_xs()
                                .text_color(rgb(RAIL))
                                .cursor_pointer()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.accept_friend_request(click_id.clone(), cx);
                                }))
                                .child(action_label),
                        )
                    })
                    .when(!is_none && !is_incoming, |element| {
                        element.child(
                            div()
                                .text_xs()
                                .text_color(rgb(MUTED))
                                .child(action_label),
                        )
                    })
            }).collect::<Vec<_>>())
            .when(!query.is_empty() && results.is_empty(), |element| {
                element.child(empty_hint("No matching users. Try another username or name."))
            })
            .child(section_heading("INCOMING REQUESTS"))
            .children(incoming.iter().map(|user| {
                let user_id = user.user_id.clone();
                let name = user.display_name.clone();
                let username = user.username.clone();
                let click_id = user_id.clone();
                div()
                    .id(format!("incoming-friend-{user_id}"))
                    .p_2()
                    .rounded_md()
                    .bg(rgb(SURFACE))
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(avatar_view(
                        self.cached_avatar_paths.get(&user_id).cloned(),
                        name.chars().next().unwrap_or('?').to_string(),
                        32.0,
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(div().text_sm().text_color(rgb(TEXT)).child(name))
                            .child(div().text_xs().text_color(rgb(MUTED)).child(format!("@{username}"))),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .id(format!("accept-friend-{user_id}"))
                            .px_2()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(ACCENT))
                            .text_xs()
                            .text_color(rgb(RAIL))
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.accept_friend_request(click_id.clone(), cx);
                            }))
                            .child("Accept"),
                    )
            }).collect::<Vec<_>>())
            .when(incoming.is_empty(), |element| {
                element.child(empty_hint("No incoming requests"))
            })
            .child(section_heading("YOUR FRIENDS"))
            .children(friends.iter().map(|user| {
                person_row(
                    user.display_name.chars().next().unwrap_or('?').to_string(),
                    user.display_name.clone(),
                    if user.username.is_empty() {
                        if user.online { "Online".into() } else { "Offline".into() }
                    } else {
                        format!("@{} · {}", user.username, if user.online { "Online" } else { "Offline" })
                    },
                    self.cached_avatar_paths.get(&user.user_id).cloned(),
                )
            }))
            .when(friends.is_empty(), |element| {
                element.child(empty_hint("You have no friends yet"))
            })
            .child(section_heading("OUTGOING REQUESTS"))
            .children(outgoing.iter().map(|user| {
                person_row(
                    user.display_name.chars().next().unwrap_or('?').to_string(),
                    user.display_name.clone(),
                    format!("@{} · Request sent", user.username),
                    self.cached_avatar_paths.get(&user.user_id).cloned(),
                )
            }))
            .when(outgoing.is_empty(), |element| {
                element.child(empty_hint("No outgoing requests"))
            })
    }

    fn group_creation_view(
        &self,
        mode: GroupCreationMode,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let friends = self.friends.clone();
        let selected = self.selected_group_members.clone();
        let is_channel = mode == GroupCreationMode::Channel;
        let heading = if is_channel { "NEW CHANNEL" } else { "NEW GROUP" };
        let submit_label = match mode {
            GroupCreationMode::Group => if self.creating_group { "Creating…" } else { "Create group" },
            GroupCreationMode::Channel => if self.creating_group { "Creating…" } else { "Create channel" },
        };

        div()
            .p_3()
            .rounded_lg()
            .border_1()
            .border_color(rgb(BORDER))
            .bg(rgb(SIDEBAR))
            .flex()
            .flex_col()
            .gap_2()
            .child(section_heading(heading))
            .child(self.login_field(
                LoginField::GroupName,
                "NAME",
                if is_channel { "Channel name" } else { "Group name" },
                &self.group_name,
                cx,
            ))
            .when(is_channel, |element| {
                let policy = self.group_post_policy.clone();
                element.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .p_2()
                        .rounded_md()
                        .bg(rgb(SURFACE))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(MUTED))
                                .child("Who can publish?"),
                        )
                        .child(
                            div()
                                .id("channel-post-policy")
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .bg(rgb(SURFACE_HOVER))
                                .text_xs()
                                .text_color(rgb(ACCENT))
                                .cursor_pointer()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.group_post_policy = if this.group_post_policy == "admins" {
                                        "members".into()
                                    } else {
                                        "admins".into()
                                    };
                                    cx.notify();
                                }))
                                .child(if policy == "admins" { "Admins only" } else { "All members" }),
                        ),
                )
            })
            .child(section_heading("INVITE FRIENDS"))
            .children(friends.iter().map(|user| {
                let user_id = user.user_id.clone();
                let name = user.display_name.clone();
                let username = user.username.clone();
                let is_selected = selected.contains(&user_id);
                let avatar = self.cached_avatar_paths.get(&user_id).cloned();
                div()
                    .id(format!("group-member-{user_id}"))
                    .p_2()
                    .rounded_md()
                    .bg(if is_selected { rgb(SURFACE_HOVER) } else { rgb(SURFACE) })
                    .border_1()
                    .border_color(if is_selected { rgb(ACCENT) } else { rgb(BORDER) })
                    .flex()
                    .items_center()
                    .gap_2()
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_group_member(user_id.clone(), cx);
                    }))
                    .child(avatar_view(
                        avatar,
                        name.chars().next().unwrap_or('?').to_string(),
                        30.0,
                    ))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(div().text_sm().text_color(rgb(TEXT)).child(name))
                            .child(div().text_xs().text_color(rgb(MUTED)).child(format!("@{username}"))),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(if is_selected { rgb(ACCENT) } else { rgb(MUTED) })
                            .child(if is_selected { "✓" } else { "○" }),
                    )
            }).collect::<Vec<_>>())
            .when(friends.is_empty(), |element| {
                element.child(empty_hint("Add friends before inviting people to a group or channel"))
            })
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(MUTED))
                    .child(format!("{} friend(s) selected", selected.len())),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .id("create-group-submit")
                            .flex_1()
                            .p_2()
                            .rounded_md()
                            .bg(if self.creating_group { rgb(SURFACE_HOVER) } else { rgb(ACCENT) })
                            .text_color(rgb(RAIL))
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| this.create_group_or_channel(cx)))
                            .child(submit_label),
                    )
                    .child(
                        div()
                            .id("create-group-cancel")
                            .p_2()
                            .rounded_md()
                            .bg(rgb(SURFACE))
                            .text_color(rgb(MUTED))
                            .text_sm()
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.group_creation_mode = None;
                                this.group_name.clear();
                                this.selected_group_members.clear();
                                this.creating_group = false;
                                cx.notify();
                            }))
                            .child("Cancel"),
                    ),
            )
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
        let creation_view = if let Some(mode) = self.group_creation_mode {
            self.group_creation_view(mode, cx).into_any_element()
        } else {
            div().into_any_element()
        };

        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .id("new-group-button")
                            .flex_1()
                            .p_2()
                            .rounded_md()
                            .bg(rgb(SURFACE_HOVER))
                            .text_color(rgb(ACCENT))
                            .text_xs()
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.group_creation_mode = Some(GroupCreationMode::Group);
                                this.group_name.clear();
                                this.selected_group_members.clear();
                                this.creating_group = false;
                                cx.notify();
                            }))
                            .child("+ Group"),
                    )
                    .child(
                        div()
                            .id("new-channel-button")
                            .flex_1()
                            .p_2()
                            .rounded_md()
                            .bg(rgb(SURFACE_HOVER))
                            .text_color(rgb(ACCENT))
                            .text_xs()
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.group_creation_mode = Some(GroupCreationMode::Channel);
                                this.group_name.clear();
                                this.selected_group_members.clear();
                                this.creating_group = false;
                                cx.notify();
                            }))
                            .child("+ Channel"),
                    ),
            )
            .child(creation_view)
            .child(section_heading("YOUR GROUPS & CHANNELS"))
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

    fn settings_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let message_policy = self.core.state().current_user.as_ref()
            .map(|user| user.message_policy.as_str())
            .unwrap_or("everyone");
        let privacy_label = if message_policy == "friends" {
            "Only friends can send you messages"
        } else {
            "Everyone can send you messages"
        };

        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .child(section_heading("PROFILE"))
            .child(self.login_field(
                LoginField::ProfileDisplayName,
                "DISPLAY NAME",
                "Your display name",
                &self.profile_display_name,
                cx,
            ))
            .child(self.login_field(
                LoginField::ProfileUsername,
                "USERNAME",
                "@username",
                &self.profile_username,
                cx,
            ))
            .child(self.login_field(
                LoginField::ProfileAbout,
                "ABOUT",
                "A short profile description",
                &self.profile_about,
                cx,
            ))
            .child(
                div()
                    .id("save-profile-button")
                    .p_2()
                    .rounded_md()
                    .bg(if self.saving_profile { rgb(SURFACE_HOVER) } else { rgb(ACCENT) })
                    .text_color(rgb(RAIL))
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| this.save_profile(cx)))
                    .child(if self.saving_profile { "Saving profile…" } else { "Save profile" }),
            )
            .child(section_heading("PRIVACY"))
            .child(
                div()
                    .p_3()
                    .rounded_md()
                    .bg(rgb(SURFACE))
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(div().text_sm().text_color(rgb(TEXT)).child("Who can message you"))
                            .child(div().text_xs().text_color(rgb(MUTED)).child(privacy_label)),
                    )
                    .child(
                        div()
                            .id("toggle-message-policy")
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(SURFACE_HOVER))
                            .text_xs()
                            .text_color(rgb(ACCENT))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| this.toggle_message_policy(cx)))
                            .child("Change"),
                    ),
            )
            .child(section_heading("APPLICATION"))
            .child(settings_row("Connection", self.connection_status.clone()))
            .child(settings_row(
                "End-to-end encryption",
                self.crypto.as_ref()
                    .map(|crypto| format!("Enabled · Matrix device {}", crypto.device_id()))
                    .unwrap_or_else(|| "Locked · sign in with your E2E recovery key".to_string()),
            ))
            .child(settings_row(
                "Appearance",
                "Native theme controls are not yet available".to_string(),
            ))
    }

    fn saved_messages_view(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let messages: Vec<_> = self.saved_messages.iter().cloned().collect();
        let is_loaded = self.saved_messages_loaded;
        let message_rows: Vec<_> = messages.iter().map(|message| {
            let id = message.id.clone();
            let created = format_saved_age(message.created_at);
            let body = message.body.clone();
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(MUTED))
                                .child(created),
                        )
                        .child(div().flex_1())
                        .child(
                            div()
                                .id(format!("delete-saved-message-{id}"))
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .bg(rgb(SURFACE))
                                .text_xs()
                                .text_color(rgb(MUTED))
                                .cursor_pointer()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.delete_saved_message(&id, cx);
                                }))
                                .child("Delete"),
                        ),
                )
                .child(message_bubble("You".into(), body, true))
        }).collect();

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
                    .flex_col()
                    .justify_center()
                    .gap_0p5()
                    .border_b_1()
                    .border_color(rgb(BORDER))
                    .child(
                        div()
                            .text_base()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(TEXT))
                            .child("Saved Messages"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(MUTED))
                            .child("Private notes · encrypted on this device"),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_6()
                    .when(!is_loaded, |element| {
                        element.child(empty_hint("Unlocking your saved notes…"))
                    })
                    .children(message_rows)
                    .when(is_loaded && messages.is_empty(), |element| {
                        element.child(empty_hint("Nothing saved yet. Add a note below."))
                    }),
            )
            .child(
                div()
                    .p_4()
                    .border_t_1()
                    .border_color(rgb(BORDER))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(self.login_field(
                        LoginField::SavedComposer,
                        "SAVE A NOTE",
                        "Write a message, reminder, or link…",
                        &self.saved_message_text,
                        cx,
                    ))
                    .child(
                        div()
                            .id("save-message-button")
                            .p_3()
                            .rounded_md()
                            .bg(if is_loaded { rgb(ACCENT) } else { rgb(SURFACE_HOVER) })
                            .text_color(rgb(RAIL))
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| this.save_saved_message(cx)))
                            .child("Save to this device"),
                    ),
            )
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
        let peer_avatar = self.cached_avatar_paths.get(peer_id).cloned();
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
                    .child(avatar_view(
                        peer_avatar,
                        display_name.chars().next().unwrap_or('?').to_string(),
                        36.0,
                    ))
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
                                let cached_preview = self.cached_attachment_paths.get(&metadata.id).cloned();
                                let has_cached_preview = cached_preview.is_some();
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
                                    .when_some(cached_preview, |element, path| {
                                        element.child(
                                            img(path)
                                                .w(px(280.0))
                                                .h(px(190.0))
                                                .object_fit(gpui::ObjectFit::Contain),
                                        )
                                    })
                                    .when(!has_cached_preview, |element| element.child(label))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.open_attachment(
                                            target_peer.clone(),
                                            target_message.clone(),
                                            metadata.clone(),
                                            cx,
                                        );
                                    }))
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

impl Drop for LarptrixShell {
    fn drop(&mut self) {
        self.media_session_id = None;
        if let Some(player) = self.media_player.take() {
            player.stop();
        }
        if let Some(path) = self.media_path.take() {
            let _ = std::fs::remove_file(path);
        }
    }
}

impl Render for LarptrixShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.core.state().current_user.is_none() {
            match self.connection_page {
                ConnectionPage::Servers => self.server_picker_view(cx).into_any_element(),
                ConnectionPage::SignIn => self.login_view(cx).into_any_element(),
            }
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
                .child(if self.active_section == Section::Saved {
                    self.saved_messages_view(cx).into_any_element()
                } else {
                    self.conversation(cx).into_any_element()
                })
                .when(self.image_viewer.is_some(), |element| {
                    element.child(self.image_viewer_overlay(cx))
                })
                .when(self.media_loading || self.media_player.is_some() || self.media_error.is_some(), |element| {
                    element.child(self.media_player_overlay(cx))
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

    fn media_player_overlay(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let is_video = self.media_is_video;
        let title = self.media_title.clone();
        let frame = self.media_frame.clone();
        let has_frame = frame.is_some();
        let is_playing = self.media_playing;
        let is_finished = self.media_finished;
        let loading = self.media_loading;
        let position = self.media_position_ms;
        let duration = self.media_duration_ms;
        let volume = self.media_volume;
        let error = self.media_error.clone();
        let error_visible = error.is_some();
        let error_text = error.unwrap_or_default();
        let segment_count = 40usize;

        div()
            .absolute()
            .inset_0()
            .occlude()
            .bg(rgb(0x10141d))
            .flex()
            .flex_col()
            .p_5()
            .gap_4()
            .child(
                div()
                    .h(px(48.0))
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(ACCENT))
                                    .child(if is_video { "VIDEO" } else { "AUDIO" }),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(TEXT))
                                    .child(title),
                            ),
                    )
                    .child(
                        div()
                            .id("close-media-player")
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(SURFACE))
                            .text_sm()
                            .text_color(rgb(TEXT))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.close_media_player();
                                cx.notify();
                            }))
                            .child("Close  ×"),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.0))
                    .rounded_lg()
                    .bg(rgb(0x080b12))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .flex()
                    .items_center()
                    .justify_center()
                    .overflow_hidden()
                    .when(is_video, |element| {
                        element.when_some(frame, |element, frame| {
                            element.child(
                                img(frame)
                                    .max_w_full()
                                    .max_h(px(620.0))
                                    .object_fit(gpui::ObjectFit::Contain),
                            )
                        })
                        .when(!has_frame && loading, |element| {
                            element.child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(MUTED))
                                    .child("Decrypting and opening video…"),
                            )
                        })
                        .when(!has_frame && !loading && !error_visible, |element| {
                            element.child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(MUTED))
                                    .child("Waiting for the first video frame…"),
                            )
                        })
                    })
                    .when(!is_video, |element| {
                        element.child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap_3()
                                .child(
                                    div()
                                        .text_color(rgb(ACCENT))
                                        .text_size(px(72.0))
                                        .child("♫"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(MUTED))
                                        .child(if loading { "Preparing audio…" } else if is_playing { "Now playing" } else { "Playback paused" }),
                                ),
                        )
                    })
                    .when(error_visible, |element| {
                        element.child(
                            div()
                                .absolute()
                                .bottom_3()
                                .left_3()
                                .right_3()
                                .p_3()
                                .rounded_md()
                                .bg(rgb(0x301d28))
                                .text_xs()
                                .text_color(rgb(RED))
                                .child(error_text),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_3()
                    .rounded_lg()
                    .bg(rgb(SIDEBAR))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .text_xs()
                            .text_color(rgb(MUTED))
                            .child(format_media_time(position))
                            .child(format_media_time(duration)),
                    )
                    .child(
                        div()
                            .id("media-seek-track")
                            .w_full()
                            .flex()
                            .gap(px(2.0))
                            .children((0..segment_count).map(|index| {
                                let target = if duration == 0 {
                                    0
                                } else {
                                    duration.saturating_mul(index as u64)
                                        / (segment_count.saturating_sub(1) as u64)
                                };
                                let played = duration > 0
                                    && (index as u64).saturating_mul(duration)
                                        <= position.saturating_mul(segment_count as u64);
                                div()
                                    .id(format!("media-seek-{index}"))
                                    .flex_1()
                                    .h(px(8.0))
                                    .rounded_sm()
                                    .bg(if played { rgb(ACCENT) } else { rgb(BORDER) })
                                    .cursor_pointer()
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.seek_media(target, cx);
                                    }))
                            })),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap_2()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .id("media-seek-back")
                                            .px_3()
                                            .py_2()
                                            .rounded_md()
                                            .bg(rgb(SURFACE))
                                            .text_color(rgb(TEXT))
                                            .text_xs()
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.seek_media_by(-10_000, cx);
                                            }))
                                            .child("−10s"),
                                    )
                                    .child(
                                        div()
                                            .id("media-play-pause")
                                            .px_4()
                                            .py_2()
                                            .rounded_md()
                                            .bg(rgb(ACCENT))
                                            .text_color(rgb(RAIL))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_sm()
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.toggle_media_playback(cx);
                                            }))
                                            .child(if is_playing { "Ⅱ  Pause" } else if is_finished { "↺  Replay" } else { "▶  Play" }),
                                    )
                                    .child(
                                        div()
                                            .id("media-seek-forward")
                                            .px_3()
                                            .py_2()
                                            .rounded_md()
                                            .bg(rgb(SURFACE))
                                            .text_color(rgb(TEXT))
                                            .text_xs()
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.seek_media_by(10_000, cx);
                                            }))
                                            .child("+10s"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .id("media-volume-down")
                                            .px_2()
                                            .py_2()
                                            .rounded_md()
                                            .bg(rgb(SURFACE))
                                            .text_color(rgb(TEXT))
                                            .text_sm()
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.change_media_volume(-0.1, cx);
                                            }))
                                            .child("−"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(MUTED))
                                            .child(format!("VOL  {:.0}%", volume * 100.0)),
                                    )
                                    .child(
                                        div()
                                            .id("media-volume-up")
                                            .px_2()
                                            .py_2()
                                            .rounded_md()
                                            .bg(rgb(SURFACE))
                                            .text_color(rgb(TEXT))
                                            .text_sm()
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.change_media_volume(0.1, cx);
                                            }))
                                            .child("+"),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn server_picker_view(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let server_entries: Vec<_> = self.saved_servers.iter().cloned().enumerate().map(|(index, server)| {
            let server_url = server.clone();
            div()
                .id(format!("saved-server-{index}"))
                .w_full()
                .p_3()
                .rounded_md()
                .border_1()
                .border_color(rgb(BORDER))
                .bg(rgb(SURFACE))
                .text_color(rgb(TEXT))
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.server_url = server_url.clone();
                    this.choose_server(cx);
                }))
                .child(server)
        }).collect();

        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(BG))
            .child(
                div()
                    .w(px(520.0))
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
                                    .child("Choose a Larptrix server"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(MUTED))
                                    .child("Connect to your own server or choose one saved on this device."),
                            ),
                    )
                    .child(self.login_field(
                        LoginField::Server,
                        "SERVER ADDRESS",
                        "https://chat.example.com",
                        &self.server_url,
                        cx,
                    ))
                    .child(
                        div()
                            .id("choose-server-button")
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
                            .on_click(cx.listener(|this, _, _, cx| this.choose_server(cx)))
                            .child("Continue"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(MUTED))
                            .child("SAVED SERVERS"),
                    )
                    .children(server_entries)
                    .when(self.saved_servers.is_empty(), |element| {
                        element.child(
                            div()
                                .text_sm()
                                .text_color(rgb(MUTED))
                                .child("No saved servers yet. Add your server address above."),
                        )
                    })
                    .child(
                        div()
                            .text_xs()
                            .text_color(if self.connection_status.contains("Invalid")
                                || self.connection_status.contains("Enter")
                                || self.connection_status.contains("must")
                            {
                                rgb(RED)
                            } else {
                                rgb(MUTED)
                            })
                            .child(self.connection_status.clone()),
                    ),
            )
    }

    fn login_view(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let is_login = self.auth_mode == AuthMode::SignIn;
        let is_register = self.auth_mode == AuthMode::Register;
        let status_lower = self.connection_status.to_ascii_lowercase();
        let status_is_error = status_lower.contains("failed")
            || status_lower.contains("invalid")
            || status_lower.contains("error")
            || status_lower.contains("taken");

        let suggestion_rows: Vec<_> = self.username_suggestions.iter().cloned().map(|username| {
            let selected = self.selected_register_username.as_deref() == Some(username.as_str());
            let click_value = username.clone();
            div()
                .id(format!("username-suggestion-{username}"))
                .p_2()
                .rounded_md()
                .border_1()
                .border_color(if selected { rgb(ACCENT) } else { rgb(BORDER) })
                .bg(if selected { rgb(SURFACE_HOVER) } else { rgb(SURFACE) })
                .text_color(if selected { rgb(TEXT) } else { rgb(MUTED) })
                .text_sm()
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.select_register_username(click_value.clone(), cx);
                }))
                .child(format!("@{username}{}", if selected { "  ✓" } else { "" }))
        }).collect();

        let server_field = self.login_field(
            LoginField::Server,
            "SERVER",
            "larptrix.duckdns.org",
            &self.server_url,
            cx,
        );

        let access_key_field = self.login_field(
            LoginField::AccessKey,
            "ACCESS KEY",
            "Paste your 64-character access key",
            &self.access_key,
            cx,
        );

        let recovery_field = self.login_field(
            LoginField::RecoveryKey,
            "OLD E2E RECOVERY KEY  ·  OPTIONAL",
            "Paste the key for your previous encrypted chats",
            &self.recovery_key,
            cx,
        );

        let email_field = self.login_field(
            LoginField::Email,
            "EMAIL",
            "you@example.com",
            &self.email,
            cx,
        );
        let password_field = self.login_field(
            LoginField::Password,
            "PASSWORD",
            "Account password",
            &self.password,
            cx,
        );

        let display_name_field = self.login_field(
            LoginField::DisplayName,
            "DISPLAY NAME",
            "How should people see you?",
            &self.display_name,
            cx,
        );

        let qr_view = self.sign_in_qr_view();

        div()
            .size_full()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .overflow_hidden()
            .bg(rgb(0x080d18))
            .child(space_background())
            .child(
                div()
                    .w(px(1040.0))
                    .max_w(px(1120.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_8()
                    .child(
                        div()
                            .w(px(650.0))
                            .p_6()
                            .rounded_xl()
                            .border_1()
                            .border_color(rgb(0x34445f))
                            .bg(rgb(0x111a2b))
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .child(pixel_l_logo())
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_0p5()
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
                                                    .child("A little corner of the universe. Your server, your space."),
                                            ),
                                    ),
                            )
                            .child(server_field)
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(
                                        div()
                                            .id("auth-tab-login")
                                            .flex_1()
                                            .p_3()
                                            .rounded_md()
                                            .bg(if is_login { rgb(SURFACE_HOVER) } else { rgb(0x0c1422) })
                                            .border_1()
                                            .border_color(if is_login { rgb(ACCENT) } else { rgb(BORDER) })
                                            .text_sm()
                                            .text_color(if is_login { rgb(TEXT) } else { rgb(MUTED) })
                                            .text_center()
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.auth_mode = AuthMode::SignIn;
                                                this.active_input = Some(LoginField::AccessKey);
                                                this.connection_status.clear();
                                                cx.notify();
                                            }))
                                            .child("Sign in"),
                                    )
                                    .child(
                                        div()
                                            .id("auth-tab-register")
                                            .flex_1()
                                            .p_3()
                                            .rounded_md()
                                            .bg(if is_register { rgb(SURFACE_HOVER) } else { rgb(0x0c1422) })
                                            .border_1()
                                            .border_color(if is_register { rgb(ACCENT) } else { rgb(BORDER) })
                                            .text_sm()
                                            .text_color(if is_register { rgb(TEXT) } else { rgb(MUTED) })
                                            .text_center()
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.auth_mode = AuthMode::Register;
                                                this.active_input = Some(LoginField::DisplayName);
                                                this.connection_status.clear();
                                                this.registration_access_key = None;
                                                cx.notify();
                                            }))
                                            .child("Create account"),
                                    ),
                            )
                            .when(is_login, |element| {
                                let key_mode = self.login_mode == LoginMode::AccessKey;
                                element
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .text_color(rgb(MUTED))
                                                    .child(if key_mode { "SIGN IN WITH ACCESS KEY" } else { "OLDER ACCOUNT" }),
                                            )
                                            .child(
                                                div()
                                                    .id("toggle-legacy-login")
                                                    .text_xs()
                                                    .text_color(rgb(ACCENT))
                                                    .cursor_pointer()
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.login_mode = if this.login_mode == LoginMode::AccessKey {
                                                            LoginMode::EmailPassword
                                                        } else {
                                                            LoginMode::AccessKey
                                                        };
                                                        this.active_input = Some(if this.login_mode == LoginMode::AccessKey {
                                                            LoginField::AccessKey
                                                        } else {
                                                            LoginField::Email
                                                        });
                                                        cx.notify();
                                                    }))
                                                    .child(if key_mode { "Use email/password" } else { "Use access key" }),
                                            ),
                                    )
                                    .when(key_mode, |element| element.child(access_key_field))
                                    .when(!key_mode, |element| element.child(email_field).child(password_field))
                                    .child(recovery_field)
                                    .child(
                                        div()
                                            .p_3()
                                            .rounded_md()
                                            .bg(rgb(0x0c1422))
                                            .border_1()
                                            .border_color(rgb(BORDER))
                                            .text_xs()
                                            .text_color(rgb(0xb4c4d9))
                                            .child("The old E2E recovery key is only needed to restore your previous encrypted conversations. Leave it empty to create a fresh E2E identity and a new recovery key on this device."),
                                    )
                                    .when_some(self.registration_access_key.clone(), |element, key| {
                                        element.child(
                                            div()
                                                .p_3()
                                                .rounded_md()
                                                .bg(rgb(0x172b32))
                                                .border_1()
                                                .border_color(rgb(ACCENT))
                                                .flex()
                                                .flex_col()
                                                .gap_1()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                                        .text_color(rgb(ACCENT))
                                                        .child("ACCOUNT CREATED · SAVE YOUR ACCESS KEY"),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(TEXT))
                                                        .child(key),
                                                ),
                                        )
                                    })
                                    .child(
                                        div()
                                            .id("connect-button")
                                            .w_full()
                                            .p_3()
                                            .rounded_md()
                                            .bg(rgb(ACCENT))
                                            .text_color(rgb(RAIL))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_sm()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _, _, cx| this.start_connection(cx)))
                                            .child(if self.connecting { "Connecting…" } else { "Sign in" }),
                                    )
                            })
                            .when(is_register, |element| {
                                element
                                    .child(display_name_field)
                                    .child(
                                        div()
                                            .p_3()
                                            .rounded_md()
                                            .bg(rgb(0x0c1422))
                                            .border_1()
                                            .border_color(rgb(BORDER))
                                            .text_xs()
                                            .text_color(rgb(0xb4c4d9))
                                            .child("Your username is generated from your display name. You do not need to choose one unless the suggested username is already taken."),
                                    )
                                    .when(!self.username_suggestions.is_empty(), |element| {
                                        element
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .text_color(rgb(MUTED))
                                                    .child(format!("@{} is already taken · choose a similar username", self.username_base)),
                                            )
                                            .children(suggestion_rows)
                                    })
                                    .when_some(self.selected_register_username.clone(), |element, username| {
                                        element.child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(ACCENT))
                                                .child(format!("Your username will be @{username}")),
                                        )
                                    })
                                    .child(
                                        div()
                                            .id("register-button")
                                            .w_full()
                                            .p_3()
                                            .rounded_md()
                                            .bg(rgb(ACCENT))
                                            .text_color(rgb(RAIL))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_sm()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _, _, cx| this.start_registration(cx)))
                                            .child(if self.registering {
                                                "Creating account…"
                                            } else if self.selected_register_username.is_some() {
                                                "Create account with selected username"
                                            } else {
                                                "Create account"
                                            }),
                                    )
                            })
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(if status_is_error { rgb(RED) } else { rgb(MUTED) })
                                    .child(self.connection_status.clone()),
                            ),
                    )
                    .child(if is_login {
                        qr_view.into_any_element()
                    } else {
                        registration_side_card().into_any_element()
                    }),
            )
    }

    fn sign_in_qr_view(&self) -> impl IntoElement {
        let access_key: String = self.access_key
            .chars()
            .filter(|ch| ch.is_ascii_hexdigit())
            .collect();
        let qr_data = if self.login_mode == LoginMode::AccessKey && access_key.len() == 64 {
            network::normalize_server_url(&self.server_url)
                .ok()
                .map(|url| format!("{}#access_key={access_key}", url))
        } else {
            None
        };
        let qr_rows = qr_data.and_then(|data| QrCode::new(data.as_bytes()).ok()).map(|code| {
            let width = code.width();
            (0..width).map(|y| {
                let cells: Vec<_> = (0..width).map(|x| {
                    div()
                        .size(px(4.0))
                        .bg(if code[(x, y)] == QrColor::Dark {
                            rgb(0x0b1220)
                        } else {
                            rgb(0xffffff)
                        })
                }).collect();
                div().flex().gap(px(0.0)).children(cells)
            }).collect::<Vec<_>>()
        });

        div()
            .w(px(260.0))
            .p_4()
            .rounded_xl()
            .border_1()
            .border_color(rgb(0x34445f))
            .bg(rgb(0x111a2b))
            .flex()
            .flex_col()
            .items_center()
            .gap_3()
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(ACCENT))
                    .child("QUICK SIGN IN"),
            )
            .child(
                div()
                    .size(px(204.0))
                    .p_2()
                    .rounded_lg()
                    .bg(rgb(0xffffff))
                    .flex()
                    .items_center()
                    .justify_center()
                    .when_some(qr_rows, |element, rows| {
                        element.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(0.0))
                                .children(rows),
                        )
                    })
                    .when(self.login_mode != LoginMode::AccessKey || access_key.len() != 64, |element| {
                        element.child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap_2()
                                .child(pixel_l_logo())
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x45536a))
                                        .text_center()
                                        .child("Enter your access key to generate a sign-in QR"),
                                ),
                        )
                    }),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(MUTED))
                    .text_center()
                    .child("Scan with your phone to open this server and sign in to the same account."),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x71819a))
                    .text_center()
                    .child("The QR contains your access key. Keep it private."),
            )
    }

}

fn server_input_value(raw: &str) -> String {
    raw.trim()
        .strip_prefix("https://")
        .or_else(|| raw.trim().strip_prefix("http://"))
        .unwrap_or(raw.trim())
        .trim_end_matches('/')
        .to_string()
}

fn space_background() -> impl IntoElement {
    let stars = [
        (32.0, 42.0, 2.0, 0x9bb7df), (120.0, 118.0, 2.0, 0x718cb5),
        (220.0, 54.0, 3.0, 0xc2d7f0), (328.0, 168.0, 2.0, 0x91abc9),
        (448.0, 82.0, 2.0, 0x718cb5), (560.0, 34.0, 2.0, 0xb5c9e6),
        (680.0, 142.0, 3.0, 0x9bb7df), (804.0, 58.0, 2.0, 0x718cb5),
        (936.0, 182.0, 2.0, 0xc2d7f0), (1088.0, 76.0, 2.0, 0x91abc9),
        (1210.0, 130.0, 2.0, 0x718cb5), (1348.0, 48.0, 3.0, 0xb5c9e6),
        (1482.0, 190.0, 2.0, 0x9bb7df), (1640.0, 96.0, 2.0, 0x718cb5),
        (1760.0, 164.0, 2.0, 0xc2d7f0), (1840.0, 58.0, 2.0, 0x91abc9),
        (74.0, 310.0, 2.0, 0x718cb5), (260.0, 370.0, 2.0, 0xb5c9e6),
        (420.0, 300.0, 3.0, 0x9bb7df), (592.0, 410.0, 2.0, 0x718cb5),
        (760.0, 330.0, 2.0, 0xc2d7f0), (930.0, 420.0, 2.0, 0x91abc9),
        (1110.0, 300.0, 2.0, 0x718cb5), (1280.0, 388.0, 3.0, 0xb5c9e6),
        (1460.0, 320.0, 2.0, 0x9bb7df), (1640.0, 420.0, 2.0, 0x718cb5),
        (1800.0, 340.0, 2.0, 0xc2d7f0), (150.0, 610.0, 2.0, 0x91abc9),
        (360.0, 540.0, 2.0, 0x718cb5), (550.0, 690.0, 3.0, 0xb5c9e6),
        (760.0, 590.0, 2.0, 0x9bb7df), (970.0, 720.0, 2.0, 0x718cb5),
        (1180.0, 560.0, 2.0, 0xc2d7f0), (1390.0, 700.0, 2.0, 0x91abc9),
        (1580.0, 590.0, 3.0, 0x718cb5), (1770.0, 720.0, 2.0, 0xb5c9e6),
        (60.0, 880.0, 2.0, 0x9bb7df), (280.0, 810.0, 2.0, 0x718cb5),
        (480.0, 950.0, 2.0, 0xc2d7f0), (700.0, 840.0, 2.0, 0x91abc9),
        (920.0, 990.0, 3.0, 0x718cb5), (1150.0, 860.0, 2.0, 0xb5c9e6),
        (1370.0, 980.0, 2.0, 0x9bb7df), (1590.0, 850.0, 2.0, 0x718cb5),
        (1800.0, 960.0, 2.0, 0xc2d7f0),
    ];
    let star_elements: Vec<_> = stars.into_iter().map(|(x, y, size, color)| {
        div()
            .absolute()
            .left(px(x))
            .top(px(y))
            .size(px(size))
            .bg(rgb(color))
    }).collect();
    let comets = vec![
        comet_l(145.0, 235.0, 2.0, 0x88c0d0),
        comet_l(520.0, 225.0, 1.5, 0x7898c2),
        comet_l(1010.0, 225.0, 2.0, 0x88c0d0),
        comet_l(1510.0, 260.0, 1.5, 0x7898c2),
        comet_l(325.0, 735.0, 1.5, 0x88c0d0),
        comet_l(875.0, 820.0, 2.0, 0x7898c2),
        comet_l(1290.0, 760.0, 1.5, 0x88c0d0),
        comet_l(1710.0, 650.0, 2.0, 0x7898c2),
    ];

    div()
        .absolute()
        .inset_0()
        .bg(rgb(0x080d18))
        .child(
            div()
                .absolute()
                .top(px(120.0))
                .left(px(260.0))
                .size(px(620.0))
                .rounded_full()
                .bg(rgb(0x0c1729)),
        )
        .child(
            div()
                .absolute()
                .top(px(360.0))
                .left(px(1120.0))
                .size(px(560.0))
                .rounded_full()
                .bg(rgb(0x0b1425)),
        )
        .children(star_elements)
        .children(comets)
}

fn comet_l(x: f32, y: f32, scale: f32, color: u32) -> impl IntoElement {
    div()
        .absolute()
        .left(px(x))
        .top(px(y))
        .flex()
        .flex_col()
        .gap(px(scale))
        .child(div().w(px(scale * 2.0)).h(px(scale * 7.0)).bg(rgb(color)))
        .child(div().w(px(scale * 8.0)).h(px(scale * 2.0)).bg(rgb(color)))
        .child(
            div()
                .absolute()
                .left(px(0.0))
                .top(px(scale * 6.0))
                .size(px(scale * 2.0))
                .bg(rgb(0xd8dee9)),
        )
}

fn pixel_l_logo() -> impl IntoElement {
    div()
        .size(px(48.0))
        .rounded_lg()
        .bg(rgb(0x88c0d0))
        .flex()
        .flex_col()
        .justify_end()
        .p(px(9.0))
        .gap(px(0.0))
        .child(div().w(px(6.0)).h(px(25.0)).bg(rgb(0x101827)))
        .child(div().w(px(26.0)).h(px(6.0)).bg(rgb(0x101827)))
}

fn registration_side_card() -> impl IntoElement {
    div()
        .w(px(260.0))
        .p_4()
        .rounded_xl()
        .border_1()
        .border_color(rgb(0x34445f))
        .bg(rgb(0x111a2b))
        .flex()
        .flex_col()
        .items_center()
        .gap_4()
        .child(
            div()
                .size(px(204.0))
                .rounded_lg()
                .bg(rgb(0x0c1422))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.0))
                        .child(comet_l(0.0, 0.0, 4.0, 0x88c0d0))
                        .child(pixel_l_logo()),
                ),
        )
        .child(
            div()
                .text_xs()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(ACCENT))
                .child("MAKE THIS SERVER YOURS"),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(MUTED))
                .text_center()
                .child("Choose a display name. Larptrix will create a unique username for you."),
        )
}

fn server_list_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("APPDATA").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("larptrix").join("servers.txt"))
}

fn load_saved_servers() -> Vec<String> {
    let Some(path) = server_list_path() else {
        return Vec::new();
    };
    let Ok(contents) = std::fs::read_to_string(path) else {
        return Vec::new();
    };

    let mut servers = Vec::new();
    for raw in contents.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let candidate = if raw.contains("://") {
            raw.to_string()
        } else {
            format!("https://{raw}")
        };
        if let Ok(url) = network::normalize_server_url(&candidate) {
            let normalized = url.to_string();
            if !servers.iter().any(|saved: &String| saved.eq_ignore_ascii_case(&normalized)) {
                servers.push(normalized);
            }
        }
        if servers.len() == 20 {
            break;
        }
    }
    servers
}

fn save_saved_servers(servers: &[String]) {
    let Some(path) = server_list_path() else {
        return;
    };
    let Some(parent) = path.parent() else {
        return;
    };
    if std::fs::create_dir_all(parent).is_err() {
        return;
    }
    let contents = servers.join("\n");
    let _ = std::fs::write(path, contents);
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

fn avatar_view(path: Option<PathBuf>, initials: String, size: f32) -> impl IntoElement {
    let has_image = path.is_some();
    div()
        .size(px(size))
        .rounded_full()
        .bg(rgb(SURFACE_HOVER))
        .flex()
        .items_center()
        .justify_center()
        .overflow_hidden()
        .when_some(path, |element, path| {
            element.child(
                img(path)
                    .w(px(size))
                    .h(px(size))
                    .object_fit(gpui::ObjectFit::Contain),
            )
        })
        .when(!has_image, |element| {
            element
                .text_sm()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(ACCENT))
                .child(initials)
        })
}

fn chat_entry(
    id: String,
    name: String,
    preview: String,
    time: String,
    avatar: Option<PathBuf>,
    selected: bool,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
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
        .child(avatar_view(
            avatar,
            name.chars().next().unwrap_or('?').to_string(),
            40.0,
        ))
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

fn format_saved_age(timestamp: i64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(timestamp);
    let elapsed = now.saturating_sub(timestamp).max(0);
    if elapsed < 60 {
        "just now".into()
    } else if elapsed < 3_600 {
        format!("{} min ago", elapsed / 60)
    } else if elapsed < 86_400 {
        format!("{} hr ago", elapsed / 3_600)
    } else {
        format!("{} days ago", elapsed / 86_400)
    }
}

fn format_media_time(milliseconds: u64) -> String {
    let total_seconds = milliseconds / 1_000;
    let hours = total_seconds / 3_600;
    let minutes = (total_seconds / 60) % 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

fn media_extension(mime: &str) -> &'static str {
    match mime {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/bmp" => "bmp",
        "image/avif" => "avif",
        "image/tiff" => "tif",
        "image/svg+xml" => "svg",
        "image/x-icon" => "ico",
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

fn person_row(initial: String, name: String, status: String, avatar: Option<PathBuf>) -> impl IntoElement {
    div()
        .p_2()
        .rounded_md()
        .flex()
        .items_center()
        .gap_3()
        .child(avatar_view(avatar, initial, 34.0))
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

#[cfg(test)]
mod media_time_tests {
    use super::format_media_time;

    #[test]
    fn formats_audio_and_video_positions() {
        assert_eq!(format_media_time(0), "0:00");
        assert_eq!(format_media_time(65_000), "1:05");
        assert_eq!(format_media_time(3_661_000), "1:01:01");
    }
}

#[cfg(test)]
mod media_tests {
    use super::media_extension;

    #[test]
    fn selects_file_extensions_for_image_audio_and_video_players() {
        assert_eq!(media_extension("image/jpeg"), "jpg");
        assert_eq!(media_extension("image/png"), "png");
        assert_eq!(media_extension("image/gif"), "gif");
        assert_eq!(media_extension("image/webp"), "webp");
        assert_eq!(media_extension("video/mp4"), "mp4");
        assert_eq!(media_extension("video/webm"), "webm");
        assert_eq!(media_extension("audio/webm"), "webm");
        assert_eq!(media_extension("audio/mpeg"), "mp3");
        assert_eq!(media_extension("audio/ogg"), "ogg");
        assert_eq!(media_extension("application/octet-stream"), "bin");
    }
}

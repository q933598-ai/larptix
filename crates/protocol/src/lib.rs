//! Wire protocol for Larptrix 0.2.
//!
//! Auth is HTTP (cookie session). The WebSocket carries chat events only.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Load history with one person.
    Open { peer_id: String },
    /// Send text and/or a previously uploaded attachment.
    Send {
        peer_id: String,
        #[serde(default)]
        body: String,
        #[serde(default)]
        attachment_id: Option<String>,
    },
    /// Relay ephemeral WebRTC signaling data to one chat peer.
    CallSignal {
        peer_id: String,
        kind: String,
        payload: serde_json::Value,
    },
    CryptoResync {
        peer_id: String,
        message_id: String,
        device_id: String,
        body: String,
        #[serde(default) ]
        attachment_id: Option<String>,
    },
    /// Deliver a freshly encrypted copy of a message to a specific device
    /// without creating another chat history entry.
    CryptoResyncResponse {
        peer_id: String,
        message_id: String,
        device_id: String,
        body: String,
        #[serde(default) ]
        attachment_id: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Welcome {
        user: UserInfo,
        users: Vec<UserInfo>,
    },
    Directory {
        users: Vec<UserInfo>,
    },
    Groups {
        groups: Vec<GroupInfo>,
    },
    Chat {
        peer: UserInfo,
        history: Vec<ChatMessage>,
    },
    Message {
        message: ChatMessage,
    },
    CallSignal {
        sender_id: String,
        kind: String,
        payload: serde_json::Value,
    },
    CryptoResync {
        requester_id: String,
        message_id: String,
        device_id: String,
        body: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        attachment_id: Option<String>,
    },
    CryptoResyncResponse {
        sender_id: String,
        message_id: String,
        device_id: String,
        body: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        attachment_id: Option<String>,
    },
    Error {
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub group_id: String,
    pub name: String,
    pub member_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub display_name: String,
    #[serde(default)]
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    pub online: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity: Option<String>,
    #[serde(default)]
    pub is_group: bool,
    #[serde(default)]
    pub e2e_enabled: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub group_member_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub recipient_id: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<AttachmentInfo>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentInfo {
    pub id: String,
    pub mime: String,
    pub url: String,
    pub name: String,
    pub size_bytes: i64,
}

pub const MAX_DISPLAY_NAME: usize = 32;
pub const MAX_BODY: usize = 12_000;
pub const MAX_EMAIL: usize = 254;
pub const MIN_PASSWORD: usize = 8;
pub const MAX_PASSWORD: usize = 128;
pub const HISTORY_LIMIT: usize = 500;
pub const MAX_IMAGE_BYTES: usize = 5 * 1024 * 1024;
pub const MAX_ATTACHMENT_BYTES: usize = 25 * 1024 * 1024;

pub fn sanitize_display_name(raw: &str) -> Result<String, &'static str> {
    let name = raw.trim();
    if name.is_empty() {
        return Err("display name is empty");
    }
    if name.chars().count() > MAX_DISPLAY_NAME {
        return Err("display name is too long");
    }
    Ok(name.to_string())
}

pub fn sanitize_email(raw: &str) -> Result<String, &'static str> {
    let email = raw.trim().to_lowercase();
    if email.is_empty() || email.len() > MAX_EMAIL {
        return Err("email is invalid");
    }
    let Some((local, domain)) = email.split_once('@') else {
        return Err("email is invalid");
    };
    if local.is_empty() || !domain.contains('.') {
        return Err("email is invalid");
    }
    Ok(email)
}

pub fn sanitize_password(raw: &str) -> Result<&str, &'static str> {
    let len = raw.chars().count();
    if len < MIN_PASSWORD {
        return Err("password must be at least 8 characters");
    }
    if len > MAX_PASSWORD {
        return Err("password is too long");
    }
    Ok(raw)
}

pub fn sanitize_body(raw: &str) -> Result<String, &'static str> {
    let body = raw.trim();
    if body.chars().count() > MAX_BODY {
        return Err("message is too long");
    }
    Ok(body.to_string())
}

pub fn avatar_url(user_id: &str) -> String {
    format!("/api/users/{user_id}/avatar")
}

pub fn attachment_url(id: &str) -> String {
    format!("/api/attachments/{id}")
}

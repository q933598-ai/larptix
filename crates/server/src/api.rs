use std::sync::Arc;

use uuid::Uuid;

use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Multipart, Path, Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use larptrix_protocol::{
    attachment_url, avatar_url, sanitize_display_name, sanitize_email, sanitize_password,
    AttachmentInfo, GroupInfo, ServerMessage, UserActivity, UserInfo, MAX_ATTACHMENT_BYTES,
    MAX_IMAGE_BYTES,
};
use serde::Deserialize;

use crate::auth::{
    access_key_hash, hash_password, new_access_key, new_qr_login_token, new_session_token,
    verify_password, COOKIE_NAME, QR_LOGIN_MS, SESSION_MS,
};
use crate::db::{DbError, UserRow};
use crate::media::{
    detect_audio, detect_image, safe_extension, safe_filename, upload_path, write_upload,
};
use crate::now_ms;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/register", post(register))
        .route("/api/register/password", post(register_password))
        .route("/api/login", post(login))
        .route("/api/qr-login/claim/{token}", post(claim_qr_login))
        .route("/api/qr-login/status/{token}", get(qr_login_status))
        .route("/api/logout", post(logout))
        .route("/api/me", get(me))
        .route("/api/me", patch(update_profile))
        .route("/api/me/activity", post(update_activity))
        .route("/api/me/activities", post(update_activities))
        .route("/api/me/settings", patch(update_message_policy))
        .route("/api/friends", get(list_friends))
        .route(
            "/api/friends/{id}",
            post(send_friend_request).delete(remove_friend),
        )
        .route("/api/friends/{id}/accept", post(accept_friend_request))
        .route("/api/users/search", get(search_users))
        .route("/api/users/{id}/profile", get(get_user_profile))
        .route(
            "/api/users/{id}/matrix-devices",
            get(list_user_matrix_devices),
        )
        .route("/api/users/{id}/music", get(get_user_music))
        .route(
            "/api/me/music",
            post(set_music)
                .delete(clear_music)
                .layer(DefaultBodyLimit::max(MAX_ATTACHMENT_BYTES + 64 * 1024)),
        )
        .route("/api/groups", get(list_groups).post(create_group))
        .route("/api/groups/{id}/members", post(add_group_member))
        .route("/api/groups/{id}/settings", patch(update_group_settings))
        .route(
            "/api/groups/{id}/avatar",
            get(get_group_avatar)
                .post(set_group_avatar)
                .layer(DefaultBodyLimit::max(MAX_IMAGE_BYTES + 64 * 1024)),
        )
        .route(
            "/api/groups/{id}/banner",
            get(get_group_banner)
                .post(set_group_banner)
                .layer(DefaultBodyLimit::max(MAX_IMAGE_BYTES + 64 * 1024)),
        )
        .route("/api/channels", post(create_channel))
        .route(
            "/api/channels/{id}/admins",
            post(add_channel_admin).delete(remove_channel_admin),
        )
        .route(
            "/api/channels/{id}/settings",
            patch(update_channel_settings),
        )
        .route("/api/me/access-key", post(create_access_key))
        .route("/api/me/qr-login", post(create_qr_login))
        .route("/api/me/qr-login/{token}", get(my_qr_login_status).post(approve_qr_login))
        .route("/api/me/sessions", get(list_sessions))
        .route("/api/me/sessions/others", delete(delete_other_sessions))
        .route("/api/me/sessions/{session_id}", delete(delete_session))
        .route(
            "/api/me/crypto-device",
            get(get_crypto_device).put(put_crypto_device),
        )
        .route(
            "/api/me/crypto-devices",
            get(list_crypto_devices).post(create_crypto_device),
        )
        .route(
            "/api/me/crypto-devices/{device_id}",
            get(get_my_crypto_device)
                .put(update_crypto_device)
                .delete(delete_crypto_device),
        )
        .route(
            "/api/me/matrix-devices/{device_id}",
            delete(delete_my_matrix_device),
        )
        .route(
            "/api/users/{id}/crypto-devices",
            get(list_user_crypto_devices),
        )
        .route(
            "/api/users/{id}/crypto-devices/{device_id}/claim-one-time-key",
            post(claim_crypto_one_time_key),
        )
        .route("/api/users/{id}/crypto-key", get(get_crypto_key))
        .route("/api/matrix/config", get(matrix_config))
        .route(
            "/api/matrix/key-backup",
            get(matrix_key_backup_get)
                .put(matrix_key_backup_put)
                .layer(DefaultBodyLimit::max(16 * 1024 * 1024 + 64 * 1024)),
        )
        .route("/api/matrix/keys/upload", post(matrix_keys_upload))
        .route("/api/matrix/keys/query", post(matrix_keys_query))
        .route("/api/matrix/keys/claim", post(matrix_keys_claim))
        .route("/api/matrix/to-device", post(matrix_send_to_device))
        .route(
            "/api/matrix/to-device/pending",
            get(matrix_pending_to_device),
        )
        .route("/api/matrix/to-device/ack", post(matrix_ack_to_device))
        .route("/api/rtc-config", get(rtc_config))
        .route("/api/users/{id}/avatar", get(user_avatar))
        .route("/api/attachments/{id}", get(get_attachment))
        .route(
            "/api/upload",
            post(upload).layer(DefaultBodyLimit::max(MAX_ATTACHMENT_BYTES + 64 * 1024)),
        )
        .route(
            "/api/me/avatar",
            post(set_avatar).layer(DefaultBodyLimit::max(MAX_IMAGE_BYTES + 64 * 1024)),
        )
        .route(
            "/api/me/banner",
            post(set_profile_banner).layer(DefaultBodyLimit::max(MAX_IMAGE_BYTES + 64 * 1024)),
        )
        .route("/api/users/{id}/banner", get(user_banner))
}

#[derive(Deserialize)]
pub struct RegisterBody {
    pub display_name: String,
    pub username: String,
}

#[derive(Deserialize)]
pub struct PasswordRegisterBody {
    pub display_name: String,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginBody {
    pub access_key: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize)]
pub struct MatrixKeysUploadBody {
    pub device_id: String,
    pub request: serde_json::Value,
}

#[derive(Deserialize)]
pub struct MatrixToDeviceBody {
    pub device_id: String,
    pub event_type: String,
    pub txn_id: String,
    pub messages: serde_json::Value,
}

#[derive(Deserialize)]
pub struct MatrixToDeviceAckBody {
    pub event_ids: Vec<i64>,
}

#[derive(Deserialize)]
pub struct MatrixKeyBackupBody {
    pub encrypted_backup: String,
}

#[derive(Deserialize)]
pub struct UpdateProfileBody {
    pub display_name: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub about: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
pub struct UpdateActivityBody {
    pub activity: String,
}

#[derive(Deserialize)]
pub struct UpdateActivitiesBody {
    #[serde(default)]
    pub activities: Vec<UserActivity>,
}

#[derive(Deserialize)]
pub struct UpdateMessagePolicyBody {
    pub message_policy: String,
}

#[derive(Deserialize)]
pub struct UserSearchQuery {
    pub q: String,
}

#[derive(Deserialize)]
pub struct CryptoDeviceBody {
    pub bundle_json: String,
    pub encrypted_state: String,
    #[serde(default)]
    pub clear_old_history: bool,
}

#[derive(Deserialize)]
pub struct CreateCryptoDeviceBody {
    pub device_id: String,
    pub bundle_json: String,
    pub encrypted_state: String,
}

#[derive(Deserialize)]
pub struct UpdateCryptoDeviceBody {
    pub bundle_json: String,
    pub encrypted_state: String,
    pub expected_version: i64,
}

async fn register(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<RegisterBody>,
) -> Result<Response, ApiError> {
    let display_name = sanitize_display_name(&body.display_name).map_err(ApiError::bad)?;
    let username = sanitize_username(&body.username).map_err(ApiError::bad)?;
    if username.is_empty() {
        return Err(ApiError::bad(
            "username is required when creating an account",
        ));
    }
    let access_key = new_access_key();
    let access_key_hash = access_key_hash(&access_key).expect("generated key is valid");
    let user = state
        .db
        .create_key_user(&display_name, &username, &access_key_hash, now_ms())
        .map_err(ApiError::from_db)?;
    broadcast_friend_directories(&state, &state.hub.online_ids());
    let mut info = public_me(&user);
    info.username = username;
    Ok(Json(serde_json::json!({
        "user": info,
        "access_key": access_key,
        "e2e_setup_required": true
    }))
    .into_response())
}

async fn register_password(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<PasswordRegisterBody>,
) -> Result<Response, ApiError> {
    let display_name = sanitize_display_name(&body.display_name).map_err(ApiError::bad)?;
    let username = sanitize_username(&body.username).map_err(ApiError::bad)?;
    if username.is_empty() {
        return Err(ApiError::bad(
            "username is required when creating an account",
        ));
    }
    let email = sanitize_email(&body.email).map_err(ApiError::bad)?;
    let password = sanitize_password(&body.password)
        .map_err(ApiError::bad)?
        .to_string();
    let password_hash = tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .map_err(|_| ApiError::internal("password hash task failed"))?
        .map_err(ApiError::internal)?;
    let user = state
        .db
        .create_user(&email, &password_hash, &display_name, &username, now_ms())
        .map_err(ApiError::from_db)?;
    let online = state.hub.online_ids();
    let users = state.db.list_users(&online).map_err(ApiError::db)?;
    state.hub.broadcast(ServerMessage::Directory { users });
    cookie_response(&state, user, device_name(&headers))
}

async fn login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<LoginBody>,
) -> Result<Response, ApiError> {
    if let Some(access_key) = body.access_key {
        let access_key_hash = access_key_hash(&access_key)
            .ok_or_else(|| ApiError::bad("access key must be 64 hexadecimal characters"))?;
        let user = state
            .db
            .user_by_access_key_hash(&access_key_hash)
            .map_err(ApiError::db)?
            .ok_or_else(|| ApiError::unauthorized("invalid access key"))?;
        return cookie_response(&state, user, device_name(&headers));
    }
    let email = sanitize_email(
        body.email
            .as_deref()
            .ok_or_else(|| ApiError::bad("email is required for legacy login"))?,
    )
    .map_err(ApiError::bad)?;
    let password = sanitize_password(
        body.password
            .as_deref()
            .ok_or_else(|| ApiError::bad("password is required for legacy login"))?,
    )
    .map_err(ApiError::bad)?;
    let user = state
        .db
        .user_by_email(&email)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::unauthorized("invalid email or password"))?;
    let hash = user.password_hash.clone();
    let password = password.to_string();
    let ok = tokio::task::spawn_blocking(move || verify_password(&password, &hash))
        .await
        .map_err(|_| ApiError::internal("verify task failed"))?;
    if !ok {
        return Err(ApiError::unauthorized("invalid email or password"));
    }
    cookie_response(&state, user, device_name(&headers))
}

fn device_name(headers: &HeaderMap) -> String {
    let ua = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("Web browser");
    let name = if ua.contains("Firefox") {
        "Firefox"
    } else if ua.contains("Edg/") {
        "Edge"
    } else if ua.contains("Chrome") {
        "Chrome"
    } else if ua.contains("Safari") {
        "Safari"
    } else if ua.contains("Electron") {
        "Larptrix desktop"
    } else {
        "Web browser"
    };
    let platform = if ua.contains("Android") {
        "Android"
    } else if ua.contains("iPhone") || ua.contains("iPad") {
        "iOS"
    } else if ua.contains("Windows") {
        "Windows"
    } else if ua.contains("Mac OS") {
        "macOS"
    } else if ua.contains("Linux") {
        "Linux"
    } else {
        "Unknown"
    };
    format!("{name} · {platform}")
}

fn public_origin(headers: &HeaderMap) -> String {
    if let Ok(value) = std::env::var("LARPTRIX_PUBLIC_URL") {
        return value.trim_end_matches('/').to_string();
    }
    let host = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("localhost");
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("https");
    format!("{proto}://{host}").trim_end_matches('/').to_string()
}

async fn create_qr_login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let token = new_qr_login_token();
    let created_at = now_ms();
    let expires_at = created_at + QR_LOGIN_MS;
    state.db.create_qr_login(&token, &user.id, expires_at, created_at).map_err(ApiError::db)?;
    let login_url = format!("{}#qr_login={token}", public_origin(&headers));
    Ok(Json(serde_json::json!({
        "token": token,
        "login_url": login_url,
        "expires_at": expires_at
    })))
}

async fn my_qr_login_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(token): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let Some((state_name, owner_id, expires_at)) =
        state.db.qr_login_state(&token, now_ms()).map_err(ApiError::db)?
    else {
        return Err(ApiError::not_found("QR login request not found"));
    };
    if owner_id != user.id {
        return Err(ApiError::unauthorized("QR login request belongs to another account"));
    }
    if expires_at <= now_ms() {
        return Ok(Json(serde_json::json!({ "state": "expired" })));
    }
    Ok(Json(serde_json::json!({ "state": state_name, "expires_at": expires_at })))
}

async fn approve_qr_login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(token): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let approved = state.db.approve_qr_login(&token, &user.id, now_ms()).map_err(ApiError::db)?;
    if !approved {
        return Err(ApiError::bad("QR login request is not ready to approve"));
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn claim_qr_login(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let _ = headers;
    let user_id = state.db.claim_qr_login(&token, now_ms()).map_err(ApiError::db)?;
    if user_id.is_none() {
        return Err(ApiError::unauthorized("invalid or expired QR login code"));
    }
    Ok(Json(serde_json::json!({ "state": "scanned" })))
}

async fn qr_login_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(token): Path<String>,
) -> Result<Response, ApiError> {
    let _ = headers;
    let Some((state_name, _, expires_at)) = state.db.qr_login_state(&token, now_ms()).map_err(ApiError::db)? else {
        return Err(ApiError::not_found("QR login request not found"));
    };
    if expires_at <= now_ms() {
        return Ok(Json(serde_json::json!({ "state": "expired" })).into_response());
    }
    if state_name != "approved" {
        return Ok(Json(serde_json::json!({ "state": state_name })).into_response());
    }
    let user_id = state.db.consume_qr_login(&token, now_ms()).map_err(ApiError::db)?
        .ok_or_else(|| ApiError::unauthorized("QR login is no longer available"))?;
    let user = state.db.user_by_id(&user_id).map_err(ApiError::db)?
        .ok_or_else(|| ApiError::unauthorized("account no longer exists"))?;
    let response = Json(public_me(&user)).into_response();
    session_response(&state, &user, device_name(&headers), response)
}

async fn list_sessions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let current_token = session_token(&headers);
    let current_id = current_token
        .as_deref()
        .and_then(|token| state.db.session_id_for_token(token).ok().flatten());
    let sessions = state.db.list_sessions(&user.id, now_ms()).map_err(ApiError::db)?;
    Ok(Json(serde_json::json!({
        "sessions": sessions.into_iter().map(|(id, created_at, expires_at, device_name)| {
            serde_json::json!({
                "id": id,
                "created_at": created_at,
                "expires_at": expires_at,
                "device_name": device_name,
                "current": current_id.as_deref() == Some(id.as_str())
            })
        }).collect::<Vec<_>>()
    })))
}

async fn delete_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    if !state.db.delete_session_by_id(&user.id, &session_id).map_err(ApiError::db)? {
        return Err(ApiError::not_found("session not found"));
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn delete_other_sessions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let current = session_token(&headers)
        .ok_or_else(|| ApiError::unauthorized("not signed in"))?;
    let removed = state.db.delete_other_sessions(&user.id, &current).map_err(ApiError::db)?;
    Ok(Json(serde_json::json!({ "ok": true, "removed": removed })))
}

async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    if let Some(token) = session_token(&headers) {
        let _ = state.db.delete_session(&token);
    }
    let mut response = Json(serde_json::json!({ "ok": true })).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_static("larptrix_session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0"),
    );
    Ok(response)
}

async fn me(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<UserInfo>, ApiError> {
    let user = require_user(&state, &headers)?;
    let mut info = public_me(&user);
    if let Some((_, username, _, _)) = state.db.profile_fields(&user.id).map_err(ApiError::db)? {
        info.username = username;
    }
    info.message_policy = state.db.message_policy(&user.id).map_err(ApiError::db)?;
    Ok(Json(info))
}

async fn update_message_policy(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<UpdateMessagePolicyBody>,
) -> Result<Json<UserInfo>, ApiError> {
    let user = require_user(&state, &headers)?;
    state
        .db
        .set_message_policy(&user.id, body.message_policy.trim())
        .map_err(ApiError::from_db)?;
    let mut info = public_me(&user);
    info.message_policy = state.db.message_policy(&user.id).map_err(ApiError::db)?;
    Ok(Json(info))
}

fn user_info_with_relationship(
    state: &AppState,
    mut user: UserInfo,
    viewer_id: &str,
) -> Result<UserInfo, ApiError> {
    user.friend_status = state
        .db
        .friend_status(viewer_id, &user.user_id)
        .map_err(ApiError::db)?;
    Ok(user)
}

fn users_from_ids(
    state: &AppState,
    ids: &[String],
    viewer_id: &str,
) -> Result<Vec<UserInfo>, ApiError> {
    let online = state.hub.online_ids();
    let all = state.db.list_users(&online).map_err(ApiError::db)?;
    ids.iter()
        .filter_map(|id| all.iter().find(|user| &user.user_id == id).cloned())
        .map(|user| user_info_with_relationship(state, user, viewer_id))
        .collect()
}

async fn list_friends(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let friends = state.db.friend_ids(&user.id).map_err(ApiError::db)?;
    let incoming = state
        .db
        .pending_friend_ids(&user.id, true)
        .map_err(ApiError::db)?;
    let outgoing = state
        .db
        .pending_friend_ids(&user.id, false)
        .map_err(ApiError::db)?;
    Ok(Json(serde_json::json!({
        "friends": users_from_ids(&state, &friends, &user.id)?,
        "incoming": users_from_ids(&state, &incoming, &user.id)?,
        "outgoing": users_from_ids(&state, &outgoing, &user.id)?,
    })))
}

async fn search_users(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<UserSearchQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let query = query
        .q
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .trim_start_matches('@')
        .to_lowercase();
    if query.is_empty() {
        return Ok(Json(serde_json::json!({ "users": [] })));
    }
    if query.chars().count() > 64 {
        return Err(ApiError::bad("search query is too long"));
    }
    let online = state.hub.online_ids();
    let mut users = state
        .db
        .list_users(&online)
        .map_err(ApiError::db)?
        .into_iter()
        .filter(|candidate| candidate.user_id != user.id)
        .filter(|candidate| {
            candidate.display_name.to_lowercase().contains(&query)
                || candidate.username.to_lowercase().contains(&query)
        })
        .map(|candidate| {
            let name = candidate.display_name.to_lowercase();
            let username = candidate.username.to_lowercase();
            let score = if username == query {
                0u8
            } else if username.starts_with(&query) {
                1
            } else if name == query {
                2
            } else if name.starts_with(&query) {
                3
            } else if username.contains(&query) {
                4
            } else {
                5
            };
            (score, candidate)
        })
        .collect::<Vec<_>>();

    users.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| {
                a.1.display_name
                    .to_lowercase()
                    .cmp(&b.1.display_name.to_lowercase())
            })
            .then_with(|| {
                a.1.username
                    .to_lowercase()
                    .cmp(&b.1.username.to_lowercase())
            })
    });

    let users = users
        .into_iter()
        .take(50)
        .map(|(_, candidate)| user_info_with_relationship(&state, candidate, &user.id))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(serde_json::json!({ "users": users })))
}

fn broadcast_friend_directories(state: &AppState, users: &[String]) {
    for id in users {
        if let Ok(uuid) = Uuid::parse_str(id) {
            let online = state.hub.online_ids();
            let friend_ids = state.db.friend_ids(id).unwrap_or_default();
            let mut directory = state.db.list_users(&online).unwrap_or_default();
            directory.retain(|user| {
                friend_ids
                    .iter()
                    .any(|friend_id| friend_id == &user.user_id)
            });
            for entry in &mut directory {
                entry.friend_status = "accepted".to_string();
            }
            let _ = state
                .hub
                .send_to(uuid, ServerMessage::Directory { users: directory });
        }
    }
}

async fn send_friend_request(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let target = state
        .db
        .user_by_id(&id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("user not found"))?;
    let status = state
        .db
        .send_friend_request(&user.id, &target.id)
        .map_err(ApiError::from_db)?;
    broadcast_friend_directories(&state, &[user.id.clone(), target.id.clone()]);
    Ok(Json(serde_json::json!({ "status": status })))
}

async fn accept_friend_request(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let target = state
        .db
        .user_by_id(&id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("user not found"))?;
    state
        .db
        .accept_friend_request(&user.id, &target.id)
        .map_err(ApiError::from_db)?;
    broadcast_friend_directories(&state, &[user.id.clone(), target.id.clone()]);
    Ok(Json(serde_json::json!({ "status": "accepted" })))
}

async fn remove_friend(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let target = state
        .db
        .user_by_id(&id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("user not found"))?;
    let removed = state
        .db
        .remove_friendship(&user.id, &target.id)
        .map_err(ApiError::db)?;
    broadcast_friend_directories(&state, &[user.id.clone(), target.id.clone()]);
    Ok(Json(serde_json::json!({ "removed": removed })))
}

async fn rtc_config(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_user(&state, &headers)?;
    let raw = std::env::var("LARPTRIX_ICE_SERVERS").ok();
    let ice_servers = parse_ice_servers(raw.as_deref()).map_err(ApiError::internal)?;
    Ok(Json(serde_json::json!({ "ice_servers": ice_servers })))
}

pub(crate) fn matrix_server_name(headers: &HeaderMap) -> Result<String, ApiError> {
    if let Ok(domain) = std::env::var("DOMAIN") {
        let domain = domain.trim().trim_end_matches('.').to_string();
        if !domain.is_empty() && !domain.contains('/') && !domain.contains(':') {
            return Ok(domain);
        }
    }

    let raw = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(header::HOST))
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::internal("server host is unavailable"))?;

    let host = raw
        .split(',')
        .next()
        .unwrap_or(raw)
        .trim()
        .trim_start_matches('[')
        .split(']')
        .next()
        .unwrap_or(raw)
        .split(':')
        .next()
        .unwrap_or(raw)
        .trim()
        .trim_end_matches('.');

    if host.is_empty() {
        return Err(ApiError::internal("server host is unavailable"));
    }
    Ok(host.to_string())
}

fn resolve_matrix_user(
    state: &AppState,
    matrix_user_id: &str,
    server_name: &str,
) -> Result<UserRow, ApiError> {
    let rest = matrix_user_id
        .strip_prefix('@')
        .ok_or_else(|| ApiError::bad("invalid Matrix user id"))?;
    let (localpart, server) = rest
        .split_once(':')
        .ok_or_else(|| ApiError::bad("invalid Matrix user id"))?;

    if localpart.is_empty() || !server.eq_ignore_ascii_case(server_name) {
        return Err(ApiError::bad("Matrix user belongs to another server"));
    }

    if let Some(user) = state.db.user_by_id(localpart).map_err(ApiError::db)? {
        return Ok(user);
    }

    state
        .db
        .user_by_username(localpart)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::bad("unknown Matrix user"))
}

async fn matrix_config(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_user(&state, &headers)?;
    Ok(Json(serde_json::json!({
        "server_name": matrix_server_name(&headers)?,
    })))
}

async fn matrix_key_backup_get(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let encrypted_backup = state
        .db
        .get_matrix_key_backup(&user.id)
        .map_err(ApiError::db)?;
    Ok(Json(serde_json::json!({
        "encrypted_backup": encrypted_backup
    })))
}

async fn matrix_key_backup_put(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<MatrixKeyBackupBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    if body.encrypted_backup.trim().is_empty() || body.encrypted_backup.len() > 16 * 1024 * 1024 {
        return Err(ApiError::bad("invalid Matrix key backup"));
    }
    state
        .db
        .upsert_matrix_key_backup(&user.id, &body.encrypted_backup)
        .map_err(ApiError::db)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn matrix_keys_upload(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<MatrixKeysUploadBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let request = body.request;

    if body.device_id.trim().is_empty() || body.device_id.len() > 255 {
        return Err(ApiError::bad("invalid Matrix device id"));
    }

    let device_keys = request
        .get("device_keys")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    if let Some(request_user_id) = device_keys
        .get("user_id")
        .and_then(serde_json::Value::as_str)
    {
        let resolved =
            resolve_matrix_user(&state, request_user_id, &matrix_server_name(&headers)?)?;
        if resolved.id != user.id {
            return Err(ApiError::bad("Matrix device belongs to another account"));
        }
    }

    if let Some(request_device_id) = device_keys
        .get("device_id")
        .and_then(serde_json::Value::as_str)
    {
        if request_device_id != body.device_id {
            return Err(ApiError::bad("Matrix device id does not match device keys"));
        }
    }

    let existing = state
        .db
        .matrix_device(&user.id, &body.device_id)
        .map_err(ApiError::db)?;

    let device_keys_json = if device_keys.as_object().is_some_and(|map| map.is_empty()) {
        existing
            .as_ref()
            .map(|device| device.device_keys_json.clone())
            .unwrap_or_else(|| {
                serde_json::json!({
                    "user_id": format!("@{}:{}", user.id, matrix_server_name(&headers).unwrap_or_else(|_| "localhost".into())),
                    "device_id": body.device_id,
                    "algorithms": ["m.olm.v1.curve25519-aes-sha2", "m.megolm.v1.aes-sha2"],
                    "keys": {},
                    "signatures": {}
                })
                .to_string()
            })
    } else {
        serde_json::to_string(&device_keys)
            .map_err(|_| ApiError::bad("invalid Matrix device keys"))?
    };

    let one_time_keys = request
        .get("one_time_keys")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let fallback_keys = request
        .get("fallback_keys")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    state
        .db
        .upsert_matrix_crypto_device(
            &user.id,
            &body.device_id,
            &device_keys_json,
            &serde_json::to_string(&one_time_keys)
                .map_err(|_| ApiError::bad("invalid Matrix one-time keys"))?,
            &serde_json::to_string(&fallback_keys)
                .map_err(|_| ApiError::bad("invalid Matrix fallback keys"))?,
        )
        .map_err(ApiError::db)?;

    let available = state
        .db
        .matrix_one_time_key_count(&body.device_id)
        .map_err(ApiError::db)?;

    Ok(Json(serde_json::json!({
        "one_time_key_counts": {
            "signed_curve25519": available
        }
    })))
}

async fn matrix_keys_query(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_user(&state, &headers)?;
    let server_name = matrix_server_name(&headers)?;
    let requested = body
        .get("device_keys")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| ApiError::bad("device_keys is required"))?;

    let mut device_keys = serde_json::Map::new();
    let mut failures = serde_json::Map::new();

    for (matrix_user_id, requested_devices) in requested {
        let user = match resolve_matrix_user(&state, matrix_user_id, &server_name) {
            Ok(user) => user,
            Err(_) => {
                failures.insert(matrix_user_id.clone(), serde_json::json!({}));
                continue;
            }
        };

        let wanted = requested_devices.as_array();
        let devices = state
            .db
            .matrix_devices_for_user(&user.id)
            .map_err(ApiError::db)?;
        let mut user_devices = serde_json::Map::new();

        for device in devices {
            if let Some(wanted) = wanted {
                if !wanted.is_empty()
                    && !wanted
                        .iter()
                        .any(|id| id.as_str() == Some(device.device_id.as_str()))
                {
                    continue;
                }
            }

            let value = serde_json::from_str::<serde_json::Value>(&device.device_keys_json)
                .map_err(|_| ApiError::internal("stored Matrix device keys are invalid"))?;
            user_devices.insert(device.device_id, value);
        }

        device_keys.insert(
            matrix_user_id.clone(),
            serde_json::Value::Object(user_devices),
        );
    }

    Ok(Json(serde_json::json!({
        "device_keys": device_keys,
        "failures": failures
    })))
}

async fn matrix_keys_claim(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_user(&state, &headers)?;
    let server_name = matrix_server_name(&headers)?;
    let requested = body
        .get("one_time_keys")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| ApiError::bad("one_time_keys is required"))?;

    let mut one_time_keys = serde_json::Map::new();

    for (matrix_user_id, requested_devices) in requested {
        let user = resolve_matrix_user(&state, matrix_user_id, &server_name)?;
        let Some(requested_devices) = requested_devices.as_object() else {
            return Err(ApiError::bad("Matrix device key requests must be objects"));
        };

        let mut user_result = serde_json::Map::new();
        for (device_id, _) in requested_devices {
            let device = state
                .db
                .matrix_device(&user.id, device_id)
                .map_err(ApiError::db)?;

            if device.is_none() {
                continue;
            }

            if let Some((key_id, value_json)) = state
                .db
                .claim_matrix_one_time_key(device_id)
                .map_err(ApiError::db)?
            {
                let value = serde_json::from_str::<serde_json::Value>(&value_json)
                    .map_err(|_| ApiError::internal("stored Matrix one-time key is invalid"))?;
                let mut device_result = serde_json::Map::new();
                device_result.insert(key_id, value);
                user_result.insert(device_id.clone(), serde_json::Value::Object(device_result));
            }
        }

        one_time_keys.insert(
            matrix_user_id.clone(),
            serde_json::Value::Object(user_result),
        );
    }

    Ok(Json(serde_json::json!({
        "one_time_keys": one_time_keys,
        "failures": {}
    })))
}

async fn matrix_send_to_device(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<MatrixToDeviceBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let sender = require_user(&state, &headers)?;

    if body.device_id.trim().is_empty()
        || body.event_type.trim().is_empty()
        || body.event_type.len() > 255
        || body.txn_id.trim().is_empty()
        || body.txn_id.len() > 255
    {
        return Err(ApiError::bad("invalid Matrix to-device request"));
    }

    state
        .db
        .matrix_device(&sender.id, &body.device_id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::bad("Matrix sending device is not registered"))?;

    let messages = body
        .messages
        .as_object()
        .ok_or_else(|| ApiError::bad("messages must be an object"))?;

    let server_name = matrix_server_name(&headers)?;

    for (matrix_user_id, devices) in messages {
        let recipient = resolve_matrix_user(&state, matrix_user_id, &server_name)?;
        let devices = devices
            .as_object()
            .ok_or_else(|| ApiError::bad("Matrix device messages must be objects"))?;

        let recipient_uuid = Uuid::parse_str(&recipient.id)
            .map_err(|_| ApiError::internal("recipient user id is invalid"))?;

        for (recipient_device_id, content) in devices {
            if recipient_device_id.is_empty() || recipient_device_id.len() > 255 {
                return Err(ApiError::bad("invalid Matrix recipient device id"));
            }

            if state
                .db
                .matrix_device(&recipient.id, recipient_device_id)
                .map_err(ApiError::db)?
                .is_none()
            {
                continue;
            }

            let content_json = serde_json::to_string(content)
                .map_err(|_| ApiError::bad("invalid Matrix to-device content"))?;

            let event_id = state
                .db
                .enqueue_matrix_to_device(
                    &recipient.id,
                    recipient_device_id,
                    &sender.id,
                    &body.device_id,
                    &body.event_type,
                    &body.txn_id,
                    &content_json,
                )
                .map_err(ApiError::db)?;

            state.hub.send_to(
                recipient_uuid,
                ServerMessage::MatrixToDevice {
                    event_id,
                    sender_id: sender.id.clone(),
                    sender_device_id: body.device_id.clone(),
                    recipient_device_id: recipient_device_id.clone(),
                    event_type: body.event_type.clone(),
                    txn_id: body.txn_id.clone(),
                    content: content.clone(),
                },
            );
        }
    }

    Ok(Json(serde_json::json!({})))
}

async fn matrix_pending_to_device(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let events = state
        .db
        .matrix_to_device_for_user(&user.id)
        .map_err(ApiError::db)?;

    let events = events
        .into_iter()
        .map(|event| {
            let content = serde_json::from_str::<serde_json::Value>(&event.content_json)
                .unwrap_or_else(|_| serde_json::json!({}));
            serde_json::json!({
                "id": event.id,
                "sender_id": event.sender_user_id,
                "sender_device_id": event.sender_device_id,
                "recipient_device_id": event.recipient_device_id,
                "event_type": event.event_type,
                "txn_id": event.txn_id,
                "content": content,
                "created_at": event.created_at
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(serde_json::json!({ "events": events })))
}

async fn matrix_ack_to_device(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<MatrixToDeviceAckBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    if body.event_ids.len() > 512 {
        return Err(ApiError::bad("too many Matrix to-device acknowledgements"));
    }
    let deleted = state
        .db
        .ack_matrix_to_device(&user.id, &body.event_ids)
        .map_err(ApiError::db)?;
    Ok(Json(serde_json::json!({ "deleted": deleted })))
}

fn parse_ice_servers(raw: Option<&str>) -> Result<serde_json::Value, &'static str> {
    const DEFAULT_ICE_SERVERS: &str = r#"[{"urls":"stun:stun.l.google.com:19302"}]"#;
    let value = match raw {
        Some(raw) if !raw.trim().is_empty() => serde_json::from_str::<serde_json::Value>(raw)
            .map_err(|_| "LARPTRIX_ICE_SERVERS must be a JSON array")?,
        _ => serde_json::from_str(DEFAULT_ICE_SERVERS).expect("built-in ICE config is valid"),
    };
    if !value.is_array() {
        return Err("LARPTRIX_ICE_SERVERS must be a JSON array");
    }
    Ok(value)
}

async fn get_crypto_device(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;

    let devices = state
        .db
        .crypto_devices_for_user(&user.id)
        .map_err(ApiError::db)?;

    let device = devices
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::not_found("E2E device is not enabled"))?;

    Ok(Json(serde_json::json!({
        "device_id": device.device_id,
        "bundle_json": device.bundle_json,
        "encrypted_state": device.encrypted_state,
        "state_version": device.state_version,
        "created_at": device.created_at,
        "updated_at": device.updated_at
    })))
}

async fn put_crypto_device(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CryptoDeviceBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;

    if body.bundle_json.len() > 16 * 1024 || body.encrypted_state.len() > 2 * 1024 * 1024 {
        return Err(ApiError::bad("encrypted device data is too large"));
    }

    let bundle: serde_json::Value = serde_json::from_str(&body.bundle_json)
        .map_err(|_| ApiError::bad("invalid public device bundle"))?;

    let device_id = bundle
        .get("device_id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| ApiError::bad("device bundle is missing its device_id"))?;

    if !bundle
        .get("fingerprint")
        .is_some_and(serde_json::Value::is_string)
    {
        return Err(ApiError::bad("device bundle is missing its fingerprint"));
    }

    let existing = state.db.crypto_device(device_id).map_err(ApiError::db)?;

    if existing.is_some() {
        return Err(ApiError::bad("crypto device already exists"));
    }

    let device = state
        .db
        .create_crypto_device(
            &user.id,
            device_id,
            &body.bundle_json,
            &body.encrypted_state,
        )
        .map_err(ApiError::db)?;

    Ok(Json(serde_json::json!({
        "e2e_enabled": true,
        "device_id": device.device_id,
        "state_version": device.state_version,
        "old_history_deleted": false
    })))
}

async fn create_crypto_device(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CreateCryptoDeviceBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;

    validate_crypto_device_payload(&body.device_id, &body.bundle_json, &body.encrypted_state)?;

    if state
        .db
        .crypto_device(&body.device_id)
        .map_err(ApiError::db)?
        .is_some()
    {
        return Err(ApiError::bad("crypto device already exists"));
    }

    let device = state
        .db
        .create_crypto_device(
            &user.id,
            &body.device_id,
            &body.bundle_json,
            &body.encrypted_state,
        )
        .map_err(ApiError::db)?;

    Ok(Json(serde_json::json!({
        "device_id": device.device_id,
        "user_id": device.user_id,
        "bundle_json": device.bundle_json,
        "state_version": device.state_version,
        "created_at": device.created_at,
        "updated_at": device.updated_at
    })))
}

async fn list_crypto_devices(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;

    let devices = state
        .db
        .crypto_devices_for_user(&user.id)
        .map_err(ApiError::db)?;

    Ok(Json(serde_json::json!({
        "devices": devices
            .into_iter()
            .map(|device| serde_json::json!({
                "device_id": device.device_id,
                "bundle_json": device.bundle_json,
                "state_version": device.state_version,
                "created_at": device.created_at,
                "updated_at": device.updated_at
            }))
            .collect::<Vec<_>>()
    })))
}

async fn get_my_crypto_device(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(device_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;

    let device = state
        .db
        .crypto_device(&device_id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("crypto device not found"))?;

    if device.user_id != user.id {
        return Err(ApiError::not_found("crypto device not found"));
    }

    Ok(Json(serde_json::json!({
        "device_id": device.device_id,
        "user_id": device.user_id,
        "bundle_json": device.bundle_json,
        "encrypted_state": device.encrypted_state,
        "state_version": device.state_version,
        "created_at": device.created_at,
        "updated_at": device.updated_at
    })))
}

async fn update_crypto_device(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(device_id): Path<String>,
    Json(body): Json<UpdateCryptoDeviceBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;

    validate_crypto_device_payload(&device_id, &body.bundle_json, &body.encrypted_state)?;

    let device = state
        .db
        .update_crypto_device(
            &user.id,
            &device_id,
            body.expected_version,
            &body.bundle_json,
            &body.encrypted_state,
        )
        .map_err(ApiError::db)?;

    let device = device.ok_or_else(|| ApiError::bad("crypto device state version conflict"))?;

    Ok(Json(serde_json::json!({
        "device_id": device.device_id,
        "state_version": device.state_version,
        "updated_at": device.updated_at
    })))
}

async fn delete_crypto_device(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(device_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;

    let deleted = state
        .db
        .delete_crypto_device(&user.id, &device_id)
        .map_err(ApiError::db)?;

    if !deleted {
        return Err(ApiError::not_found("crypto device not found"));
    }

    Ok(Json(serde_json::json!({
        "deleted": true,
        "device_id": device_id
    })))
}

async fn delete_my_matrix_device(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(device_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let deleted = state
        .db
        .delete_matrix_crypto_device(&user.id, &device_id)
        .map_err(ApiError::db)?;
    if !deleted {
        return Err(ApiError::not_found("Matrix device not found"));
    }
    Ok(Json(serde_json::json!({
        "deleted": true,
        "device_id": device_id
    })))
}

async fn list_user_crypto_devices(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_user(&state, &headers)?;

    let devices = state
        .db
        .crypto_devices_for_user(&id)
        .map_err(ApiError::db)?;

    let bundles = devices
        .into_iter()
        .map(|device| {
            serde_json::from_str::<serde_json::Value>(&device.bundle_json)
                .map_err(|_| ApiError::internal("stored public device bundle is invalid"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(serde_json::json!({
        "devices": bundles
    })))
}

fn validate_crypto_device_payload(
    device_id: &str,
    bundle_json: &str,
    encrypted_state: &str,
) -> Result<(), ApiError> {
    if device_id.is_empty() || device_id.len() > 256 {
        return Err(ApiError::bad("invalid device_id"));
    }

    if bundle_json.len() > 16 * 1024 || encrypted_state.len() > 2 * 1024 * 1024 {
        return Err(ApiError::bad("encrypted device data is too large"));
    }

    let bundle: serde_json::Value = serde_json::from_str(bundle_json)
        .map_err(|_| ApiError::bad("invalid public device bundle"))?;

    let bundle_device_id = bundle
        .get("device_id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| ApiError::bad("device bundle is missing its device_id"))?;

    if bundle_device_id != device_id {
        return Err(ApiError::bad("device_id does not match device bundle"));
    }

    if !bundle
        .get("fingerprint")
        .is_some_and(serde_json::Value::is_string)
    {
        return Err(ApiError::bad("device bundle is missing its fingerprint"));
    }

    Ok(())
}

async fn claim_crypto_one_time_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((id, device_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_user(&state, &headers)?;

    let device = state
        .db
        .crypto_device(&device_id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("crypto device not found"))?;

    if device.user_id != id {
        return Err(ApiError::not_found("crypto device not found"));
    }

    let bundle = state
        .db
        .claim_crypto_one_time_key(&device_id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::bad("no one-time keys available for this device"))?;

    Ok(Json(serde_json::json!({
        "device_id": device_id,
        "bundle": serde_json::from_str::<serde_json::Value>(&bundle)
            .map_err(|_| ApiError::internal("claimed public device bundle is invalid"))?,
    })))
}

async fn get_crypto_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_user(&state, &headers)?;

    let devices = state
        .db
        .crypto_devices_for_user(&id)
        .map_err(ApiError::db)?;

    if devices.is_empty() {
        return Err(ApiError::not_found("peer has not enabled E2E"));
    }

    let bundles = devices
        .into_iter()
        .map(|device| {
            serde_json::from_str::<serde_json::Value>(&device.bundle_json)
                .map_err(|_| ApiError::internal("stored public device bundle is invalid"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(serde_json::json!({
        "devices": bundles
    })))
}

async fn create_access_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let access_key = new_access_key();
    let key_hash = access_key_hash(&access_key).expect("generated key is valid");
    state
        .db
        .set_access_key_hash(&user.id, &key_hash)
        .map_err(ApiError::db)?;
    Ok(Json(serde_json::json!({ "access_key": access_key })))
}

async fn update_profile(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<UpdateProfileBody>,
) -> Result<Json<UserInfo>, ApiError> {
    let user = require_user(&state, &headers)?;
    let display_name = sanitize_display_name(&body.display_name).map_err(ApiError::bad)?;
    let username = sanitize_username(&body.username).map_err(ApiError::bad)?;
    let about = body.about.trim();
    if about.chars().count() > 160 {
        return Err(ApiError::bad("about must be 160 characters or fewer"));
    }

    let mut tags = Vec::new();
    for raw in body.tags {
        let tag = raw.trim();
        if tag.is_empty() {
            continue;
        }
        if tag.chars().count() > 24 {
            return Err(ApiError::bad("profile tags must be 24 characters or fewer"));
        }
        if !tags
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(tag))
        {
            tags.push(tag.to_string());
        }
        if tags.len() >= 8 {
            break;
        }
    }
    let tags_json = serde_json::to_string(&tags)
        .map_err(|_| ApiError::internal("could not encode profile tags"))?;

    state
        .db
        .update_profile(&user.id, &display_name, &username, about, &tags_json)
        .map_err(ApiError::from_db)?;
    let online = state.hub.online_ids();
    let users = state.db.list_users(&online).map_err(ApiError::db)?;
    state.hub.broadcast(ServerMessage::Directory { users });
    let activities = state
        .db
        .profile_activities(&user.id)
        .map_err(ApiError::db)?;
    let mut info = public_me(&user);
    info.display_name = display_name;
    info.username = username;
    info.tags = tags;
    info.activity = activities.first().map(|item| item.name.clone());
    info.activities = activities;
    Ok(Json(info))
}

async fn update_activities(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<UpdateActivitiesBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let mut activities = Vec::new();

    for raw in body.activities {
        let kind = raw.kind.trim().to_ascii_lowercase();
        if !matches!(kind.as_str(), "custom" | "music") {
            return Err(ApiError::bad("unsupported activity type"));
        }
        let name = raw.name.trim();
        if name.is_empty() || name.chars().count() > 120 {
            return Err(ApiError::bad(
                "activity name must be between 1 and 120 characters",
            ));
        }
        if raw.details.chars().count() > 160 {
            return Err(ApiError::bad(
                "activity details must be 160 characters or fewer",
            ));
        }
        for url in [raw.url.as_ref(), raw.image_url.as_ref()]
            .into_iter()
            .flatten()
        {
            if !url.starts_with("https://") || url.chars().count() > 1000 {
                return Err(ApiError::bad(
                    "activity links must use HTTPS and be 1000 characters or fewer",
                ));
            }
        }

        let candidate = UserActivity {
            kind,
            name: name.to_string(),
            details: raw.details.trim().to_string(),
            url: raw
                .url
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            image_url: raw
                .image_url
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
        };

        if !activities
            .iter()
            .any(|item: &UserActivity| item.kind == candidate.kind && item.name == candidate.name)
        {
            activities.push(candidate);
        }

        if activities.len() >= 5 {
            break;
        }
    }

    state
        .db
        .set_activities(&user.id, &activities)
        .map_err(ApiError::db)?;
    let users = state
        .db
        .list_users(&state.hub.online_ids())
        .map_err(ApiError::db)?;
    state.hub.broadcast(ServerMessage::Directory { users });

    Ok(Json(serde_json::json!({
        "activity": activities.first().map(|item| item.name.clone()).unwrap_or_default(),
        "activities": activities,
    })))
}

async fn update_activity(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<UpdateActivityBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let activity = body.activity.trim();
    if activity.chars().count() > 80 {
        return Err(ApiError::bad("activity must be 80 characters or fewer"));
    }
    state
        .db
        .set_activity(&user.id, activity)
        .map_err(ApiError::db)?;
    let users = state
        .db
        .list_users(&state.hub.online_ids())
        .map_err(ApiError::db)?;
    state.hub.broadcast(ServerMessage::Directory { users });
    Ok(Json(serde_json::json!({ "activity": activity })))
}

async fn list_user_matrix_devices(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_user(&state, &headers)?;
    let user = state
        .db
        .user_by_id(&id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("user not found"))?;

    let devices = state
        .db
        .matrix_devices_for_user(&user.id)
        .map_err(ApiError::db)?
        .into_iter()
        .map(|device| serde_json::json!({ "device_id": device.device_id }))
        .collect::<Vec<_>>();

    Ok(Json(serde_json::json!({ "devices": devices })))
}

async fn get_user_profile(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_user(&state, &headers)?;
    let (display_name, username, about, avatar_id) = state
        .db
        .profile_fields(&id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("user not found"))?;
    let profile_tags = state.db.profile_tags(&id).map_err(ApiError::db)?;
    let server = matrix_server_name(&headers)?;
    let music = state
        .db
        .music_track(&id)
        .map_err(ApiError::db)?
        .map(|(_, name)| {
            serde_json::json!({
                "url": format!("/api/users/{id}/music"),
                "name": name,
            })
        });
    let online = state
        .hub
        .online_ids()
        .iter()
        .any(|online_id| online_id == &id);
    let activities = if online {
        state.db.profile_activities(&id).map_err(ApiError::db)?
    } else {
        Vec::new()
    };
    let activity = activities
        .first()
        .map(|item| item.name.clone())
        .unwrap_or_default();
    Ok(Json(serde_json::json!({
        "user_id": id,
        "display_name": display_name,
        "username": username,
        "about": about,
        "activity": activity,
        "activities": activities,
        "tags": profile_tags,
        "server": server,
        "avatar_url": avatar_id.map(|_| avatar_url(&id)),
        "banner_url": state.db.profile_banner(&id).map_err(ApiError::db)?.map(|_| format!("/api/users/{}/banner", id)),
        "music": music,
    })))
}

async fn get_user_music(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    require_user(&state, &headers)?;
    let (attachment_id, _) = state
        .db
        .music_track(&id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("no profile music"))?;
    file_response(&state, &attachment_id)
}

async fn set_music(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let saved = save_audio(&state, &user.id, multipart).await?;
    if let Some((old_id, old_ext)) = state
        .db
        .set_music_track(&user.id, Some(&saved.id))
        .map_err(ApiError::db)?
    {
        let _ = std::fs::remove_file(upload_path(&state.upload_dir, &old_id, &old_ext));
    }
    Ok(Json(serde_json::json!({
        "name": saved.name,
        "url": format!("/api/users/{}/music", user.id),
    })))
}

async fn clear_music(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    if let Some((id, ext)) = state
        .db
        .set_music_track(&user.id, None)
        .map_err(ApiError::db)?
    {
        let _ = std::fs::remove_file(upload_path(&state.upload_dir, &id, &ext));
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn list_groups(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    let user = require_user(&state, &headers)?;
    let groups = state.db.groups_for_user(&user.id).map_err(ApiError::db)?;
    Ok(Json(groups.into_iter().map(group_json).collect()))
}

#[derive(Deserialize)]
pub struct CreateGroupBody {
    pub name: String,
    pub member_ids: Vec<String>,
}

async fn create_group(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CreateGroupBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let name = sanitize_display_name(&body.name).map_err(ApiError::bad)?;
    if body.member_ids.is_empty() || body.member_ids.len() > 31 {
        return Err(ApiError::bad("choose between 1 and 31 other group members"));
    }
    let mut members = body.member_ids;
    members.sort();
    members.dedup();
    if members.iter().any(|member_id| member_id == &user.id) {
        return Err(ApiError::bad("the group creator is added automatically"));
    }
    for member_id in &members {
        if state
            .db
            .user_by_id(member_id)
            .map_err(ApiError::db)?
            .is_none()
        {
            return Err(ApiError::bad("a selected group member does not exist"));
        }
        if !state
            .db
            .are_friends(&user.id, member_id)
            .map_err(ApiError::db)?
        {
            return Err(ApiError::bad("only friends can be added to groups"));
        }
        if state
            .db
            .matrix_devices_for_user(member_id)
            .map_err(ApiError::db)?
            .is_empty()
        {
            return Err(ApiError::bad(
                "all group members must finish Matrix E2E device setup first",
            ));
        }
    }
    if state
        .db
        .matrix_devices_for_user(&user.id)
        .map_err(ApiError::db)?
        .is_empty()
    {
        return Err(ApiError::bad(
            "finish Matrix E2E device setup before creating a group",
        ));
    }
    let group = state
        .db
        .create_group(&user.id, &name, &members)
        .map_err(ApiError::from_db)?;
    for member_id in &group.member_ids {
        let groups = state
            .db
            .groups_for_user(member_id)
            .map_err(ApiError::db)?
            .into_iter()
            .map(group_info)
            .collect();
        if let Ok(recipient) = member_id.parse() {
            state
                .hub
                .send_to(recipient, ServerMessage::Groups { groups });
        }
    }
    Ok(Json(group_json(group)))
}

#[derive(Deserialize)]
pub struct CreateChannelBody {
    pub name: String,
    #[serde(default)]
    pub member_ids: Vec<String>,
    #[serde(default = "default_channel_policy")]
    pub post_policy: String,
}

fn default_channel_policy() -> String {
    "admins".to_string()
}

async fn create_channel(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CreateChannelBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let name = sanitize_display_name(&body.name).map_err(ApiError::bad)?;
    if body.member_ids.len() > 31 {
        return Err(ApiError::bad(
            "a channel can contain at most 31 invited members",
        ));
    }
    if body.post_policy != "admins" && body.post_policy != "members" {
        return Err(ApiError::bad("invalid channel posting policy"));
    }
    if state
        .db
        .matrix_devices_for_user(&user.id)
        .map_err(ApiError::db)?
        .is_empty()
    {
        return Err(ApiError::bad(
            "finish Matrix E2E device setup before creating a channel",
        ));
    }
    let mut members = body.member_ids;
    members.sort();
    members.dedup();
    if members.iter().any(|id| id == &user.id) {
        return Err(ApiError::bad("the channel creator is added automatically"));
    }
    for member_id in &members {
        if state
            .db
            .user_by_id(member_id)
            .map_err(ApiError::db)?
            .is_none()
        {
            return Err(ApiError::bad("a selected channel member does not exist"));
        }
        if !state
            .db
            .are_friends(&user.id, member_id)
            .map_err(ApiError::db)?
        {
            return Err(ApiError::bad("only friends can be added to channels"));
        }
        if state
            .db
            .matrix_devices_for_user(member_id)
            .map_err(ApiError::db)?
            .is_empty()
        {
            return Err(ApiError::bad(
                "all channel members must finish Matrix E2E device setup first",
            ));
        }
    }
    let channel = state
        .db
        .create_channel(&user.id, &name, &members, &body.post_policy)
        .map_err(ApiError::from_db)?;
    for member_id in &channel.member_ids {
        let groups = state
            .db
            .groups_for_user(member_id)
            .map_err(ApiError::db)?
            .into_iter()
            .map(group_info)
            .collect();
        if let Ok(recipient) = member_id.parse() {
            state
                .hub
                .send_to(recipient, ServerMessage::Groups { groups });
        }
    }
    Ok(Json(group_json(channel)))
}

#[derive(Deserialize)]
pub struct AddChannelAdminBody {
    pub user_id: String,
}

async fn add_channel_admin(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<AddChannelAdminBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let channel = state
        .db
        .group(&id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("channel not found"))?;
    if !channel.is_channel || !channel.admin_ids.iter().any(|admin| admin == &user.id) {
        return Err(ApiError::bad("only channel admins can manage admins"));
    }
    if !channel
        .member_ids
        .iter()
        .any(|member| member == &body.user_id)
    {
        return Err(ApiError::bad(
            "the new admin must already be a channel member",
        ));
    }
    state
        .db
        .add_channel_admin(&id, &body.user_id)
        .map_err(ApiError::from_db)?;
    Ok(Json(
        serde_json::json!({ "ok": true, "channel_id": id, "user_id": body.user_id }),
    ))
}

async fn remove_channel_admin(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<AddChannelAdminBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let channel = state
        .db
        .group(&id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("channel not found"))?;
    if !channel.is_channel || !channel.admin_ids.iter().any(|admin| admin == &user.id) {
        return Err(ApiError::bad("only channel admins can manage admins"));
    }
    if body.user_id == channel.admin_ids.first().map(String::as_str).unwrap_or("") {
        return Err(ApiError::bad("the channel creator must remain an admin"));
    }
    if body.user_id == user.id && channel.admin_ids.len() <= 1 {
        return Err(ApiError::bad("the channel must keep at least one admin"));
    }
    state
        .db
        .remove_channel_admin(&id, &body.user_id)
        .map_err(ApiError::from_db)?;
    Ok(Json(
        serde_json::json!({ "ok": true, "channel_id": id, "user_id": body.user_id }),
    ))
}

#[derive(Deserialize)]
pub struct UpdateChannelSettingsBody {
    pub post_policy: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateGroupSettingsBody {
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

async fn update_group_settings(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<UpdateGroupSettingsBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let group = state
        .db
        .group(&id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("group not found"))?;
    if group.is_channel {
        return Err(ApiError::bad("use channel settings for a channel"));
    }
    if group.admin_ids.first().map(String::as_str) != Some(user.id.as_str()) {
        return Err(ApiError::bad(
            "only the group creator can change group settings",
        ));
    }
    let name = body
        .name
        .as_deref()
        .map(|value| sanitize_display_name(value).map_err(ApiError::bad))
        .transpose()?
        .unwrap_or(group.name.clone());
    let description = body.description.unwrap_or(group.description.clone());
    if description.chars().count() > 512 {
        return Err(ApiError::bad("group description is too long"));
    }
    state
        .db
        .update_group_profile(&id, &name, &description)
        .map_err(ApiError::db)?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "group_id": id,
        "name": name,
        "description": description,
    })))
}

async fn set_group_avatar(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    multipart: Multipart,
) -> Result<Json<serde_json::Value>, ApiError> {
    set_group_media(&state, &headers, &id, multipart, "avatar_id").await
}

async fn set_group_banner(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    multipart: Multipart,
) -> Result<Json<serde_json::Value>, ApiError> {
    set_group_media(&state, &headers, &id, multipart, "banner_id").await
}

async fn set_group_media(
    state: &AppState,
    headers: &HeaderMap,
    id: &str,
    multipart: Multipart,
    column: &str,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(state, headers)?;
    let group = state
        .db
        .group(id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("group not found"))?;
    let can_edit = if group.is_channel {
        group.admin_ids.iter().any(|admin| admin == &user.id)
    } else {
        group.admin_ids.first().map(String::as_str) == Some(user.id.as_str())
    };
    if !can_edit {
        return Err(ApiError::bad(
            "you do not have permission to change this group media",
        ));
    }
    let saved = save_image(state, &user.id, multipart).await?;
    let old = state
        .db
        .set_group_media(id, column, Some(&saved.id))
        .map_err(ApiError::db)?;
    if let Some(old_id) = old {
        if let Ok(Some(old_file)) = state.db.attachment(&old_id) {
            let _ =
                std::fs::remove_file(upload_path(&state.upload_dir, &old_file.id, &old_file.ext));
        }
    }
    let url = format!(
        "/api/groups/{id}/{}",
        if column == "avatar_id" {
            "avatar"
        } else {
            "banner"
        }
    );
    if column == "avatar_id" {
        Ok(Json(serde_json::json!({ "avatar_url": url })))
    } else {
        Ok(Json(serde_json::json!({ "banner_url": url })))
    }
}

async fn get_group_avatar(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    get_group_media(&state, &headers, &id, "avatar_id").await
}

async fn get_group_banner(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    get_group_media(&state, &headers, &id, "banner_id").await
}

async fn get_group_media(
    state: &AppState,
    headers: &HeaderMap,
    id: &str,
    column: &str,
) -> Result<Response, ApiError> {
    let user = require_user(state, headers)?;
    let group = state
        .db
        .group(id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("group not found"))?;
    if !group.member_ids.iter().any(|member| member == &user.id) {
        return Err(ApiError::not_found("group media not found"));
    }
    let attachment_id = match column {
        "avatar_id" => group.avatar_id,
        "banner_id" => group.banner_id,
        _ => None,
    };
    let attachment_id =
        attachment_id.ok_or_else(|| ApiError::not_found("group media not found"))?;
    file_response(state, &attachment_id)
}

async fn update_channel_settings(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<UpdateChannelSettingsBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let channel = state
        .db
        .group(&id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("channel not found"))?;
    if !channel.is_channel || !channel.admin_ids.iter().any(|admin| admin == &user.id) {
        return Err(ApiError::bad(
            "only channel admins can change channel settings",
        ));
    }
    state
        .db
        .set_channel_post_policy(&id, &body.post_policy)
        .map_err(ApiError::from_db)?;

    let name = body
        .name
        .as_deref()
        .map(|value| sanitize_display_name(value).map_err(ApiError::bad))
        .transpose()?
        .unwrap_or(channel.name.clone());
    let description = body.description.unwrap_or(channel.description.clone());
    if description.chars().count() > 512 {
        return Err(ApiError::bad("channel description is too long"));
    }
    state
        .db
        .update_group_profile(&id, &name, &description)
        .map_err(ApiError::db)?;

    Ok(Json(serde_json::json!({
        "ok": true,
        "channel_id": id,
        "post_policy": body.post_policy,
        "name": name,
        "description": description,
    })))
}

#[derive(Deserialize)]
pub struct AddGroupMemberBody {
    pub user_id: String,
}

async fn add_group_member(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<AddGroupMemberBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    if body.user_id == user.id {
        return Err(ApiError::bad("you are already in this group"));
    }
    let group = state
        .db
        .group(&id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("group not found"))?;
    if group.is_channel {
        if !group.admin_ids.iter().any(|admin| admin == &user.id) {
            return Err(ApiError::bad("only channel admins can add members"));
        }
    } else if group.admin_ids.first().map(String::as_str) != Some(user.id.as_str()) {
        return Err(ApiError::bad("only the group creator can add members"));
    }
    if state
        .db
        .user_by_id(&body.user_id)
        .map_err(ApiError::db)?
        .is_none()
    {
        return Err(ApiError::bad("user not found"));
    }
    if !state
        .db
        .are_friends(&user.id, &body.user_id)
        .map_err(ApiError::db)?
    {
        return Err(ApiError::bad(
            "only friends can be added to groups or channels",
        ));
    }
    if state
        .db
        .matrix_devices_for_user(&body.user_id)
        .map_err(ApiError::db)?
        .is_empty()
    {
        return Err(ApiError::bad(
            "the user must finish Matrix E2E device setup first",
        ));
    }
    let group = state
        .db
        .group(&id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("group not found"))?;
    if group.member_ids.len() >= 32 {
        return Err(ApiError::bad("groups can contain at most 32 members"));
    }
    state
        .db
        .add_group_member(&id, &body.user_id)
        .map_err(ApiError::from_db)?;
    for member_id in group
        .member_ids
        .iter()
        .chain(std::iter::once(&body.user_id))
    {
        if let Ok(recipient) = member_id.parse() {
            let groups = state
                .db
                .groups_for_user(member_id)
                .map_err(ApiError::db)?
                .into_iter()
                .map(group_info)
                .collect();
            state
                .hub
                .send_to(recipient, ServerMessage::Groups { groups });
        }
    }
    Ok(Json(
        serde_json::json!({ "ok": true, "group_id": id, "user_id": body.user_id }),
    ))
}

fn group_info(group: crate::db::GroupRow) -> GroupInfo {
    let subscriber_count = group.member_ids.len();
    GroupInfo {
        group_id: group.id.clone(),
        name: group.name,
        member_ids: group.member_ids,
        is_channel: group.is_channel,
        admin_ids: group.admin_ids,
        post_policy: group.post_policy,
        description: group.description,
        avatar_url: group
            .avatar_id
            .map(|_| format!("/api/groups/{}/avatar", group.id)),
        banner_url: group
            .banner_id
            .map(|_| format!("/api/groups/{}/banner", group.id)),
        subscriber_count,
    }
}

fn group_json(group: crate::db::GroupRow) -> serde_json::Value {
    let avatar_url = group
        .avatar_id
        .as_ref()
        .map(|_| format!("/api/groups/{}/avatar", group.id));
    let banner_url = group
        .banner_id
        .as_ref()
        .map(|_| format!("/api/groups/{}/banner", group.id));
    serde_json::json!({
        "group_id": group.id,
        "name": group.name,
        "member_ids": group.member_ids,
        "is_channel": group.is_channel,
        "admin_ids": group.admin_ids,
        "post_policy": group.post_policy,
        "description": group.description,
        "avatar_url": avatar_url,
        "banner_url": banner_url,
        "subscriber_count": group.member_ids.len(),
    })
}

fn sanitize_username(raw: &str) -> Result<String, &'static str> {
    let username = raw.trim().trim_start_matches('@').to_ascii_lowercase();
    if username.is_empty() {
        return Ok(String::new());
    }
    if !(3..=24).contains(&username.len())
        || !username
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err("username must be 3-24 characters: letters, numbers, and underscores");
    }
    Ok(username)
}

async fn upload(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Result<Json<AttachmentInfo>, ApiError> {
    let user = require_user(&state, &headers)?;
    let saved = save_attachment(&state, &user.id, multipart).await?;
    Ok(Json(AttachmentInfo {
        url: attachment_url(&saved.id),
        id: saved.id,
        mime: saved.mime,
        name: saved.name,
        size_bytes: saved.size_bytes,
    }))
}

async fn set_profile_banner(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = require_user(&state, &headers)?;
    let saved = save_image(&state, &user.id, multipart).await?;
    let old = state
        .db
        .set_profile_banner(&user.id, Some(&saved.id))
        .map_err(ApiError::db)?;
    if let Some(old_id) = old {
        if let Ok(Some(old_file)) = state.db.attachment(&old_id) {
            let _ =
                std::fs::remove_file(upload_path(&state.upload_dir, &old_file.id, &old_file.ext));
        }
    }
    Ok(Json(serde_json::json!({
        "banner_url": format!("/api/users/{}/banner", user.id),
    })))
}

async fn user_banner(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    require_user(&state, &headers)?;
    let attachment_id = state
        .db
        .profile_banner(&id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("no profile banner"))?;
    file_response(&state, &attachment_id)
}

async fn set_avatar(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Result<Json<UserInfo>, ApiError> {
    let user = require_user(&state, &headers)?;
    let saved = save_image(&state, &user.id, multipart).await?;
    state
        .db
        .set_avatar(&user.id, &saved.id)
        .map_err(ApiError::db)?;
    let mut info = public_me(&user);
    info.avatar_url = Some(avatar_url(&user.id));
    Ok(Json(info))
}

async fn user_avatar(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    require_user(&state, &headers)?;
    let user = state
        .db
        .user_by_id(&id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("user not found"))?;
    let avatar_id = user
        .avatar_id
        .ok_or_else(|| ApiError::not_found("no avatar"))?;
    file_response(&state, &avatar_id)
}

async fn get_attachment(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let user = require_user(&state, &headers)?;
    let allowed = state
        .db
        .can_view_attachment(&user.id, &id)
        .map_err(ApiError::db)?;
    if !allowed {
        return Err(ApiError::not_found("attachment not found"));
    }
    file_response(&state, &id)
}

fn file_response(state: &AppState, id: &str) -> Result<Response, ApiError> {
    let row = state
        .db
        .attachment(id)
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::not_found("attachment not found"))?;
    let path = upload_path(&state.upload_dir, &row.id, &row.ext);
    let bytes = std::fs::read(&path).map_err(|_| ApiError::not_found("file missing"))?;
    let mut response = Response::new(Body::from(bytes));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&row.mime)
            .unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=3600"),
    );
    let disposition = if row.mime.starts_with("image/") || row.mime.starts_with("audio/") {
        "inline"
    } else {
        "attachment"
    };
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static(disposition),
    );
    response.headers_mut().insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    Ok(response)
}

struct SavedAttachment {
    id: String,
    mime: String,
    name: String,
    size_bytes: i64,
}

async fn save_attachment(
    state: &AppState,
    owner_id: &str,
    mut multipart: Multipart,
) -> Result<SavedAttachment, ApiError> {
    let field = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::bad("invalid upload"))?
        .ok_or_else(|| ApiError::bad("missing file"))?;
    let name = safe_filename(field.file_name());
    let bytes = field
        .bytes()
        .await
        .map_err(|_| ApiError::bad("invalid upload"))?;
    if bytes.len() > MAX_ATTACHMENT_BYTES {
        return Err(ApiError::bad("file is too large (25 MiB maximum)"));
    }
    let kind = detect_image(&bytes).or_else(|| detect_audio(&bytes));
    let (mime, ext) = kind
        .map(|kind| (kind.mime, kind.ext.to_string()))
        .unwrap_or(("application/octet-stream", safe_extension(&name)));
    let size_bytes = bytes.len() as i64;
    let id = state
        .db
        .insert_attachment(owner_id, mime, &ext, &name, size_bytes, now_ms())
        .map_err(ApiError::db)?;
    write_upload(&state.upload_dir, &id, &ext, &bytes).map_err(ApiError::internal)?;
    Ok(SavedAttachment {
        id,
        mime: mime.to_string(),
        name,
        size_bytes,
    })
}

async fn save_image(
    state: &AppState,
    owner_id: &str,
    mut multipart: Multipart,
) -> Result<SavedAttachment, ApiError> {
    let field = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::bad("invalid upload"))?
        .ok_or_else(|| ApiError::bad("missing file"))?;
    let name = safe_filename(field.file_name());
    let bytes = field
        .bytes()
        .await
        .map_err(|_| ApiError::bad("invalid upload"))?;
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(ApiError::bad("image is too large"));
    }
    let kind = detect_image(&bytes).ok_or_else(|| ApiError::bad("only jpeg, png, gif, webp"))?;
    let size_bytes = bytes.len() as i64;
    let created_at = now_ms();
    let id = state
        .db
        .insert_attachment(owner_id, kind.mime, kind.ext, &name, size_bytes, created_at)
        .map_err(ApiError::db)?;
    write_upload(&state.upload_dir, &id, kind.ext, &bytes).map_err(ApiError::internal)?;
    Ok(SavedAttachment {
        id,
        mime: kind.mime.to_string(),
        name,
        size_bytes,
    })
}

async fn save_audio(
    state: &AppState,
    owner_id: &str,
    mut multipart: Multipart,
) -> Result<SavedAttachment, ApiError> {
    let field = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::bad("invalid upload"))?
        .ok_or_else(|| ApiError::bad("missing file"))?;
    let name = safe_filename(field.file_name());
    let bytes = field
        .bytes()
        .await
        .map_err(|_| ApiError::bad("invalid upload"))?;
    if bytes.len() > MAX_ATTACHMENT_BYTES {
        return Err(ApiError::bad("file is too large (25 MiB maximum)"));
    }
    let kind = detect_audio(&bytes).ok_or_else(|| ApiError::bad("unsupported audio format"))?;
    let size_bytes = bytes.len() as i64;
    let id = state
        .db
        .insert_attachment(owner_id, kind.mime, kind.ext, &name, size_bytes, now_ms())
        .map_err(ApiError::db)?;
    write_upload(&state.upload_dir, &id, kind.ext, &bytes).map_err(ApiError::internal)?;
    Ok(SavedAttachment {
        id,
        mime: kind.mime.to_string(),
        name,
        size_bytes,
    })
}

pub fn require_user(state: &AppState, headers: &HeaderMap) -> Result<UserRow, ApiError> {
    let token = session_token(headers).ok_or_else(|| ApiError::unauthorized("not signed in"))?;
    state
        .db
        .user_by_session(&token, now_ms())
        .map_err(ApiError::db)?
        .ok_or_else(|| ApiError::unauthorized("not signed in"))
}

pub fn session_token(headers: &HeaderMap) -> Option<String> {
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    cookie.split(';').find_map(|part| {
        let (name, value) = part.trim().split_once('=')?;
        (name == COOKIE_NAME).then(|| value.to_string())
    })
}

fn cookie_response(
    state: &AppState,
    user: UserRow,
    device_name: String,
) -> Result<Response, ApiError> {
    let mut info = public_me(&user);
    if let Some((_, username, _, _)) = state.db.profile_fields(&user.id).map_err(ApiError::db)? {
        info.username = username;
    }
    let response = Json(info).into_response();
    session_response(state, &user, device_name, response)
}

fn session_response(
    state: &AppState,
    user: &UserRow,
    device_name: String,
    mut response: Response,
) -> Result<Response, ApiError> {
    let token = new_session_token();
    let session_id = Uuid::new_v4().to_string();
    let created_at = now_ms();
    let expires_at = created_at + SESSION_MS;
    state
        .db
        .create_session(&token, &user.id, expires_at, &session_id, created_at, &device_name)
        .map_err(ApiError::db)?;
    let secure = std::env::var("LARPTRIX_COOKIE_SECURE")
        .is_ok_and(|value| matches!(value.as_str(), "1" | "true" | "yes"));
    let cookie = session_cookie(&token, SESSION_MS / 1000, secure);
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|_| ApiError::internal("cookie"))?,
    );
    Ok(response)
}

fn session_cookie(token: &str, max_age: i64, secure: bool) -> String {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!("{COOKIE_NAME}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure_flag}")
}

fn public_me(user: &UserRow) -> UserInfo {
    UserInfo {
        user_id: user.id.clone(),
        display_name: user.display_name.clone(),
        username: String::new(),
        email: (!user.email.ends_with("@key.larptrix.invalid")).then(|| user.email.clone()),
        online: true,
        avatar_url: user.avatar_id.as_ref().map(|_| avatar_url(&user.id)),
        activity: None,
        is_group: false,
        is_channel: false,
        admin_ids: Vec::new(),
        post_policy: String::new(),
        e2e_enabled: false,
        friend_status: "self".to_string(),
        message_policy: "everyone".to_string(),
        group_member_ids: Vec::new(),
        group_description: String::new(),
        group_avatar_url: None,
        group_banner_url: None,
        subscriber_count: 0,
        tags: Vec::new(),
        activities: Vec::new(),
    }
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad(message: impl ToString) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.to_string(),
        }
    }

    fn unauthorized(message: impl ToString) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.to_string(),
        }
    }

    fn not_found(message: impl ToString) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.to_string(),
        }
    }

    fn service_unavailable(message: impl ToString) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            message: message.to_string(),
        }
    }

    fn internal(message: impl ToString) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.to_string(),
        }
    }

    fn db(err: rusqlite::Error) -> Self {
        tracing::error!("db: {err}");
        Self::internal("database error")
    }

    fn from_db(err: DbError) -> Self {
        match err {
            DbError::EmailTaken => Self {
                status: StatusCode::CONFLICT,
                message: "email already registered".into(),
            },
            DbError::UsernameTaken => Self {
                status: StatusCode::CONFLICT,
                message: "username is already taken".into(),
            },
            DbError::BadRequest(msg) => Self::bad(msg),
            DbError::Sqlite(err) => Self::db(err),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({ "error": self.message }));
        (self.status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        create_access_key, login, parse_ice_servers, put_crypto_device, register,
        register_password, sanitize_username, session_cookie, CryptoDeviceBody, LoginBody,
        PasswordRegisterBody, RegisterBody,
    };
    use crate::db::Database;
    use crate::hub::Hub;
    use crate::AppState;
    use axum::body::to_bytes;
    use axum::extract::State;
    use axum::http::{header, HeaderMap, HeaderValue};
    use axum::Json;
    use std::path::Path;
    use std::sync::Arc;

    #[test]
    fn secure_cookie_flag_can_be_enabled_for_tls_deployments() {
        let cookie = session_cookie("session-token", 3600, true);
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=Lax"));
        assert!(cookie.ends_with("Secure"));
    }

    #[test]
    fn usernames_are_normalized_and_restricted_to_safe_characters() {
        assert_eq!(sanitize_username(" @Alice_42 ").unwrap(), "alice_42");
        assert_eq!(sanitize_username("").unwrap(), "");
        assert!(sanitize_username("ab").is_err());
        assert!(sanitize_username("alice.name").is_err());
    }

    #[test]
    fn rtc_config_defaults_to_stun_and_rejects_invalid_values() {
        let default = parse_ice_servers(None).unwrap();
        assert_eq!(default[0]["urls"], "stun:stun.l.google.com:19302");
        assert!(parse_ice_servers(Some("not-json")).is_err());
        assert!(parse_ice_servers(Some("{} ")).is_err());
        assert_eq!(
            parse_ice_servers(Some("[]")).unwrap(),
            serde_json::json!([])
        );
    }

    #[tokio::test]
    async fn registration_and_key_login_work_without_email() {
        let state = Arc::new(AppState {
            db: Database::open(Path::new(":memory:")).unwrap(),
            hub: Hub::new(),
            upload_dir: Path::new("data/uploads").to_path_buf(),
            group_calls: std::sync::Mutex::new(std::collections::HashMap::new()),
            presence: std::sync::Mutex::new(std::collections::HashMap::new()),
        });
        let response = register(
            State(state.clone()),
            Json(RegisterBody {
                display_name: "Key User".into(),
                username: "key_user".into(),
            }),
        )
        .await
        .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert!(!response
            .headers()
            .contains_key(axum::http::header::SET_COOKIE));
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(result["access_key"].is_string());
        let access_key = result["access_key"].as_str().unwrap().to_string();
        assert_eq!(access_key.len(), 71);
        assert!(result["user"]["email"].is_null());
        let user_id = result["user"]["user_id"].as_str().unwrap();

        let response = login(
            State(state),
            Json(LoginBody {
                access_key: Some(access_key),
                email: None,
                password: None,
            }),
        )
        .await
        .unwrap();
        assert!(response
            .headers()
            .contains_key(axum::http::header::SET_COOKIE));
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let user: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(user["user_id"], user_id);
        assert!(user["email"].is_null());
    }

    #[tokio::test]
    async fn password_registration_creates_a_session_and_hashes_the_password() {
        let state = Arc::new(AppState {
            db: Database::open(Path::new(":memory:")).unwrap(),
            hub: Hub::new(),
            upload_dir: Path::new("data/uploads").to_path_buf(),
            group_calls: std::sync::Mutex::new(std::collections::HashMap::new()),
            presence: std::sync::Mutex::new(std::collections::HashMap::new()),
        });
        let response = register_password(
            State(state.clone()),
            Json(PasswordRegisterBody {
                display_name: "Password User".into(),
                username: "password_user".into(),
                email: "Password@Example.test".into(),
                password: "a-strong-test-password".into(),
            }),
        )
        .await
        .unwrap();
        assert!(response
            .headers()
            .contains_key(axum::http::header::SET_COOKIE));
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let user: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(user["email"], "password@example.test");
        let stored = state
            .db
            .user_by_email("password@example.test")
            .unwrap()
            .unwrap();
        assert_ne!(stored.password_hash, "a-strong-test-password");
        assert!(crate::auth::verify_password(
            "a-strong-test-password",
            &stored.password_hash
        ));
    }

    #[tokio::test]
    async fn first_e2e_setup_preserves_history_without_a_delete_confirmation() {
        let state = Arc::new(AppState {
            db: Database::open(Path::new(":memory:")).unwrap(),
            hub: Hub::new(),
            upload_dir: Path::new("data/uploads").to_path_buf(),
            group_calls: std::sync::Mutex::new(std::collections::HashMap::new()),
            presence: std::sync::Mutex::new(std::collections::HashMap::new()),
        });
        let user = state
            .db
            .create_key_user("E2E User", "e2e_user", "hash", crate::now_ms())
            .unwrap();
        state
            .db
            .create_session(
                "e2e-session",
                &user.id,
                crate::now_ms() + 60_000,
                "e2e-test-session",
                crate::now_ms(),
                "Test browser",
            )
            .unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("larptrix_session=e2e-session"),
        );
        let response = put_crypto_device(
            State(state.clone()),
            headers,
            Json(CryptoDeviceBody {
                bundle_json:
                    r#"{"version":2,"device_id":"test-device-1","fingerprint":"fingerprint"}"#
                        .into(),
                encrypted_state: "encrypted-state".into(),
                clear_old_history: false,
            }),
        )
        .await
        .unwrap();
        assert_eq!(response.0["old_history_deleted"], false);
        assert!(state.db.crypto_device("test-device-1").unwrap().is_some());
    }

    #[tokio::test]
    async fn legacy_account_can_issue_and_use_key_without_losing_password() {
        let state = Arc::new(AppState {
            db: Database::open(Path::new(":memory:")).unwrap(),
            hub: Hub::new(),
            upload_dir: Path::new("data/uploads").to_path_buf(),
            group_calls: std::sync::Mutex::new(std::collections::HashMap::new()),
            presence: std::sync::Mutex::new(std::collections::HashMap::new()),
        });
        let old_user = state
            .db
            .create_user(
                "legacy@example.test",
                "existing-password-hash",
                "Legacy User",
                "legacy_user",
                crate::now_ms(),
            )
            .unwrap();
        let token = "legacy-session-token";
        state
            .db
            .create_session(token, &old_user.id, crate::now_ms() + 60_000)
            .unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(&format!("larptrix_session={token}")).unwrap(),
        );

        let response = create_access_key(State(state.clone()), headers)
            .await
            .unwrap();
        let result = response.0;
        let access_key = result["access_key"].as_str().unwrap().to_string();
        let unchanged_user = state
            .db
            .user_by_email("legacy@example.test")
            .unwrap()
            .unwrap();
        assert_eq!(unchanged_user.id, old_user.id);
        assert_eq!(unchanged_user.password_hash, "existing-password-hash");

        let response = login(
            State(state),
            Json(LoginBody {
                access_key: Some(access_key),
                email: None,
                password: None,
            }),
        )
        .await
        .unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let user: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(user["user_id"], old_user.id);
        assert_eq!(user["email"], "legacy@example.test");
    }
}

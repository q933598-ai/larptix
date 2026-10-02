use std::sync::Arc;

use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use larptrix_protocol::{
    attachment_url, avatar_url, sanitize_display_name, sanitize_email, sanitize_password,
    AttachmentInfo, GroupInfo, ServerMessage, UserInfo, MAX_ATTACHMENT_BYTES, MAX_IMAGE_BYTES,
};
use serde::Deserialize;

use crate::auth::{
    access_key_hash, hash_password, new_access_key, new_session_token, verify_password,
    COOKIE_NAME, SESSION_MS,
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
        .route("/api/logout", post(logout))
        .route("/api/me", get(me))
        .route("/api/me", patch(update_profile))
        .route("/api/me/activity", post(update_activity))
        .route("/api/users/{id}/profile", get(get_user_profile))
        .route("/api/users/{id}/music", get(get_user_music))
        .route(
            "/api/me/music",
            post(set_music)
                .delete(clear_music)
                .layer(DefaultBodyLimit::max(MAX_ATTACHMENT_BYTES + 64 * 1024)),
        )
        .route("/api/groups", get(list_groups).post(create_group))
        .route("/api/me/access-key", post(create_access_key))
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
            "/api/users/{id}/crypto-devices",
            get(list_user_crypto_devices),
        )
        .route(
            "/api/users/{id}/crypto-devices/{device_id}/claim-one-time-key",
            post(claim_crypto_one_time_key),
        )
        .route("/api/users/{id}/crypto-key", get(get_crypto_key))
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
}

#[derive(Deserialize)]
pub struct RegisterBody {
    pub display_name: String,
}

#[derive(Deserialize)]
pub struct PasswordRegisterBody {
    pub display_name: String,
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
pub struct UpdateProfileBody {
    pub display_name: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub about: String,
}

#[derive(Deserialize)]
pub struct UpdateActivityBody {
    pub activity: String,
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
    Json(body): Json<RegisterBody>,
) -> Result<Response, ApiError> {
    let display_name = sanitize_display_name(&body.display_name).map_err(ApiError::bad)?;
    let access_key = new_access_key();
    let access_key_hash = access_key_hash(&access_key).expect("generated key is valid");
    let user = state
        .db
        .create_key_user(&display_name, &access_key_hash, now_ms())
        .map_err(ApiError::from_db)?;
    let online = state.hub.online_ids();
    let users = state.db.list_users(&online).map_err(ApiError::db)?;
    state.hub.broadcast(ServerMessage::Directory { users });
    Ok(Json(serde_json::json!({
        "user": public_me(&user),
        "access_key": access_key
    }))
    .into_response())
}

async fn register_password(
    State(state): State<Arc<AppState>>,
    Json(body): Json<PasswordRegisterBody>,
) -> Result<Response, ApiError> {
    let display_name = sanitize_display_name(&body.display_name).map_err(ApiError::bad)?;
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
        .create_user(&email, &password_hash, &display_name, now_ms())
        .map_err(ApiError::from_db)?;
    let online = state.hub.online_ids();
    let users = state.db.list_users(&online).map_err(ApiError::db)?;
    state.hub.broadcast(ServerMessage::Directory { users });
    cookie_response(&state, user)
}

async fn login(
    State(state): State<Arc<AppState>>,
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
        return cookie_response(&state, user);
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
    cookie_response(&state, user)
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
    Ok(Json(info))
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
    state
        .db
        .update_profile(&user.id, &display_name, &username, about)
        .map_err(ApiError::from_db)?;
    let online = state.hub.online_ids();
    let users = state.db.list_users(&online).map_err(ApiError::db)?;
    state.hub.broadcast(ServerMessage::Directory { users });
    let mut info = public_me(&user);
    info.display_name = display_name;
    info.username = username;
    Ok(Json(info))
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
    let activity = if online {
        state.db.activity(&id).map_err(ApiError::db)?
    } else {
        String::new()
    };
    Ok(Json(serde_json::json!({
        "user_id": id,
        "display_name": display_name,
        "username": username,
        "about": about,
        "activity": activity,
        "avatar_url": avatar_id.map(|_| avatar_url(&id)),
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
            .user_has_crypto_devices(member_id)
            .map_err(ApiError::db)?
        {
            return Err(ApiError::bad("all group members must set up E2E first"));
        }
    }
    if !state
        .db
        .user_has_crypto_devices(&user.id)
        .map_err(ApiError::db)?
    {
        return Err(ApiError::bad("set up E2E before creating a group"));
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
            .map(|item| GroupInfo {
                group_id: item.id,
                name: item.name,
                member_ids: item.member_ids,
            })
            .collect();
        if let Ok(recipient) = member_id.parse() {
            state
                .hub
                .send_to(recipient, ServerMessage::Groups { groups });
        }
    }
    Ok(Json(group_json(group)))
}

fn group_json(group: crate::db::GroupRow) -> serde_json::Value {
    serde_json::json!({
        "group_id": group.id,
        "name": group.name,
        "member_ids": group.member_ids,
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

fn cookie_response(state: &AppState, user: UserRow) -> Result<Response, ApiError> {
    let mut info = public_me(&user);
    if let Some((_, username, _, _)) = state.db.profile_fields(&user.id).map_err(ApiError::db)? {
        info.username = username;
    }
    let response = Json(info).into_response();
    session_response(state, &user, response)
}

fn session_response(
    state: &AppState,
    user: &UserRow,
    mut response: Response,
) -> Result<Response, ApiError> {
    let token = new_session_token();
    let expires_at = now_ms() + SESSION_MS;
    state
        .db
        .create_session(&token, &user.id, expires_at)
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
        e2e_enabled: false,
        group_member_ids: Vec::new(),
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
        });
        let response = register(
            State(state.clone()),
            Json(RegisterBody {
                display_name: "Key User".into(),
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
        });
        let response = register_password(
            State(state.clone()),
            Json(PasswordRegisterBody {
                display_name: "Password User".into(),
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
        });
        let user = state
            .db
            .create_key_user("E2E User", "hash", crate::now_ms())
            .unwrap();
        state
            .db
            .create_session("e2e-session", &user.id, crate::now_ms() + 60_000)
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
        });
        let old_user = state
            .db
            .create_user(
                "legacy@example.test",
                "existing-password-hash",
                "Legacy User",
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

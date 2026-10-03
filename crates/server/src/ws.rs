use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use larptrix_protocol::{
    avatar_url, sanitize_body, ChatMessage, ClientMessage, GroupInfo, ServerMessage, UserInfo,
    HISTORY_LIMIT,
};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::db::{CryptoResyncRequestRow, CryptoResyncResponseRow, DbError, UserRow};
use crate::hub::Outbound;
use crate::now_ms;
use crate::AppState;

pub async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    user: UserRow,
    server_name: String,
) {
    let user_id = match Uuid::parse_str(&user.id) {
        Ok(id) => id,
        Err(_) => return,
    };
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    let writer = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let text = match serde_json::to_string(&msg) {
                Ok(text) => text,
                Err(err) => {
                    tracing::error!("serialize: {err}");
                    continue;
                }
            };
            if sink.send(Message::Text(text.into())).await.is_err() {
                break;
            }
        }
    });

    state
        .hub
        .join(user_id, user.display_name.clone(), tx.clone());
    let _ = tx.send(ServerMessage::Welcome {
        user: me_info(&state, &user),
        users: directory(&state, &user.id),
    });
    let groups = state
        .db
        .groups_for_user(&user.id)
        .unwrap_or_default()
        .into_iter()
        .map(|group| GroupInfo {
            group_id: group.id,
            name: group.name,
            member_ids: group.member_ids,
        })
        .collect();
    let _ = tx.send(ServerMessage::Groups { groups });
    if let Ok(events) = state.db.matrix_to_device_for_user(&user.id) {
        for event in events {
            let content = serde_json::from_str::<serde_json::Value>(&event.content_json)
                .unwrap_or_else(|_| serde_json::json!({}));
            let _ = tx.send(ServerMessage::MatrixToDevice {
                event_id: event.id,
                sender_id: event.sender_user_id,
                sender_device_id: event.sender_device_id,
                recipient_device_id: event.recipient_device_id,
                event_type: event.event_type,
                txn_id: event.txn_id,
                content,
            });
        }
    }

    if let Ok(requests) = state.db.crypto_resync_requests_for_user(&user.id) {
        for request in requests {
            send_crypto_resync_request(&tx, request);
        }
    }

    if let Ok(responses) = state.db.crypto_resync_responses_for_user(&user.id) {
        for response in responses {
            send_crypto_resync_response(&tx, response);
        }
    }
    state.hub.broadcast(ServerMessage::Directory {
        users: directory(&state, ""),
    });
    tracing::info!(user_id = %user.id, "connected");

    while let Some(frame) = stream.next().await {
        let Ok(frame) = frame else {
            break;
        };
        match frame {
            Message::Text(text) => match serde_json::from_str::<ClientMessage>(&text) {
                Ok(ClientMessage::Ping) => {
                    tracing::debug!(user_id = %user.id, "websocket heartbeat");
                    let _ = tx.send(ServerMessage::Pong);
                }
                Ok(ClientMessage::Open { peer_id }) => {
                    if let Err(err) = open_chat(&state, &tx, &user, &peer_id) {
                        send_error(&tx, "bad_open", err);
                    }
                }
                Ok(ClientMessage::Send {
                    peer_id,
                    body,
                    attachment_id,
                }) => {
                    if let Err(err) =
                        send_dm(&state, &user, &peer_id, body, attachment_id, &server_name)
                    {
                        send_error(&tx, "bad_send", err);
                    }
                }
                Ok(ClientMessage::Delete { peer_id, message_id }) => {
                    if let Err(err) = delete_message(&state, &user, &peer_id, &message_id) {
                        send_error(&tx, "bad_delete", err);
                    }
                }
                Ok(ClientMessage::CallSignal {
                    peer_id,
                    kind,
                    payload,
                }) => {
                    if let Err(err) = relay_call_signal(&state, &user, &peer_id, &kind, payload) {
                        send_error(&tx, "bad_call_signal", err);
                    }
                }
                Ok(ClientMessage::CryptoResync {
                    peer_id,
                    message_id,
                    device_id,
                    body,
                    attachment_id,
                }) => {
                    if let Err(err) = relay_crypto_resync(
                        &state,
                        &user,
                        &peer_id,
                        &message_id,
                        &device_id,
                        &body,
                        attachment_id,
                    ) {
                        send_error(&tx, "bad_crypto_resync", err);
                    }
                }
                Ok(ClientMessage::CryptoResyncResponse {
                    peer_id,
                    message_id,
                    device_id,
                    ciphertext,
                }) => {
                    if let Err(err) = relay_crypto_resync_response(
                        &state,
                        &user,
                        &peer_id,
                        &message_id,
                        &device_id,
                        &ciphertext,
                    ) {
                        send_error(&tx, "bad_crypto_resync_response", err);
                    }
                }
                Ok(ClientMessage::CryptoResyncResponseAck {
                    peer_id,
                    message_id,
                    device_id,
                }) => {
                    if let Err(err) =
                        ack_crypto_resync_response(&state, &user, &peer_id, &message_id, &device_id)
                    {
                        send_error(&tx, "bad_crypto_resync_response_ack", err);
                    }
                }
                Err(err) => send_error(&tx, "bad_json", err.to_string()),
            },
            Message::Close(_) => break,
            _ => {}
        }
    }

    state.hub.leave(user_id, &tx);
    drop(tx);
    state.hub.broadcast(ServerMessage::Directory {
        users: directory(&state, ""),
    });
    tracing::info!(user_id = %user.id, "disconnected");
    let _ = writer.await;
}

fn send_crypto_resync_request(tx: &Outbound, request: CryptoResyncRequestRow) {
    let _ = tx.send(ServerMessage::CryptoResync {
        requester_id: request.requester_user_id,
        message_id: request.message_id,
        device_id: request.device_id,
        body: request.body,
        attachment_id: request.attachment_id,
    });
}

fn send_crypto_resync_response(tx: &Outbound, response: CryptoResyncResponseRow) {
    let _ = tx.send(ServerMessage::CryptoResyncResponse {
        sender_id: response.sender_user_id,
        message_id: response.message_id,
        device_id: response.device_id,
        ciphertext: response.ciphertext,
    });
}

fn relay_crypto_resync(
    state: &AppState,
    user: &UserRow,
    peer_id: &str,
    message_id: &str,
    device_id: &str,
    body: &str,
    attachment_id: Option<String>,
) -> Result<(), String> {
    if peer_id == user.id {
        return Err("cannot request E2E recovery from yourself".into());
    }
    if message_id.is_empty() || device_id.is_empty() {
        return Err("crypto recovery is missing message or device id".into());
    }
    if body.chars().count() > larptrix_protocol::MAX_BODY {
        return Err("crypto recovery payload is too large".into());
    }
    let _: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| "crypto recovery contains invalid encrypted message".to_string())?;

    if !state
        .db
        .dm_message_is_between(message_id, peer_id, &user.id)
        .map_err(|err| err.to_string())?
    {
        return Err("crypto recovery request does not match the original DM".into());
    }

    let peer = state
        .db
        .user_by_id(peer_id)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "unknown peer".to_string())?;
    let own_devices = state
        .db
        .crypto_devices_for_user(&user.id)
        .map_err(|err| err.to_string())?;
    if !own_devices
        .iter()
        .any(|device| device.device_id == device_id)
    {
        return Err("recovery target device does not belong to requester".into());
    }
    let recipient = Uuid::parse_str(&peer.id).map_err(|_| "invalid peer id".to_string())?;

    state
        .db
        .enqueue_crypto_resync_request(
            &user.id,
            &peer.id,
            message_id,
            device_id,
            body,
            attachment_id.as_deref(),
        )
        .map_err(|err| err.to_string())?;

    if state.hub.online_ids().iter().all(|id| id != &peer.id) {
        tracing::info!(
            requester = %user.id,
            peer = %peer.id,
            message_id = %message_id,
            device_id = %device_id,
            "crypto recovery request queued for offline peer"
        );
        return Ok(());
    }

    tracing::info!(
        requester = %user.id,
        peer = %peer.id,
        message_id = %message_id,
        device_id = %device_id,
        "crypto recovery request received"
    );

    let delivered = state.hub.send_to(
        recipient,
        ServerMessage::CryptoResync {
            requester_id: user.id.clone(),
            message_id: message_id.to_string(),
            device_id: device_id.to_string(),
            body: body.to_string(),
            attachment_id,
        },
    );

    tracing::info!(
        requester = %user.id,
        peer = %peer.id,
        message_id = %message_id,
        delivered_connections = delivered,
        "crypto recovery request relayed"
    );

    Ok(())
}

fn relay_crypto_resync_response(
    state: &AppState,
    user: &UserRow,
    peer_id: &str,
    message_id: &str,
    device_id: &str,
    ciphertext: &str,
) -> Result<(), String> {
    if peer_id == user.id {
        return Err("cannot send E2E recovery response to yourself".into());
    }
    if message_id.is_empty() || device_id.is_empty() {
        return Err("crypto recovery response is missing message or device id".into());
    }
    if ciphertext.chars().count() > larptrix_protocol::MAX_BODY {
        return Err("crypto recovery response payload is too large".into());
    }

    let envelope: serde_json::Value = serde_json::from_str(ciphertext)
        .map_err(|_| "crypto recovery response contains invalid ciphertext envelope".to_string())?;
    if envelope.get("version").and_then(serde_json::Value::as_u64) != Some(1)
        || !matches!(
            envelope
                .get("message_type")
                .and_then(serde_json::Value::as_str),
            Some("message" | "prekey")
        )
        || envelope
            .get("ciphertext")
            .and_then(serde_json::Value::as_str)
            .is_none()
    {
        return Err("crypto recovery response must contain a v1 ciphertext".into());
    }

    if !state
        .db
        .dm_message_is_between(message_id, &user.id, peer_id)
        .map_err(|err| err.to_string())?
    {
        return Err("crypto recovery response does not match the original DM".into());
    }

    let peer = state
        .db
        .user_by_id(peer_id)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "unknown peer".to_string())?;

    let peer_devices = state
        .db
        .crypto_devices_for_user(&peer.id)
        .map_err(|err| err.to_string())?;

    if !peer_devices
        .iter()
        .any(|device| device.device_id == device_id)
    {
        return Err("recovery response target device does not belong to recipient".into());
    }

    let recipient = Uuid::parse_str(&peer.id).map_err(|_| "invalid peer id".to_string())?;

    if !state
        .db
        .delete_crypto_resync_request(&peer.id, &user.id, message_id, device_id)
        .map_err(|err| err.to_string())?
    {
        return Err("crypto recovery request is no longer pending".into());
    }

    state
        .db
        .enqueue_crypto_resync_response(&peer.id, &user.id, message_id, device_id, ciphertext)
        .map_err(|err| err.to_string())?;

    if state.hub.online_ids().iter().all(|id| id != &peer.id) {
        tracing::info!(
            sender = %user.id,
            peer = %peer.id,
            message_id = %message_id,
            device_id = %device_id,
            "crypto recovery response queued for offline peer"
        );
        return Ok(());
    }

    let delivered = state.hub.send_to(
        recipient,
        ServerMessage::CryptoResyncResponse {
            sender_id: user.id.clone(),
            message_id: message_id.to_string(),
            device_id: device_id.to_string(),
            ciphertext: ciphertext.to_string(),
        },
    );

    tracing::info!(
        sender = %user.id,
        peer = %peer.id,
        message_id = %message_id,
        device_id = %device_id,
        delivered_connections = delivered,
        "crypto recovery response relayed"
    );

    if delivered == 0 {
        return Err("peer connection disappeared before recovery response delivery".into());
    }

    Ok(())
}

fn ack_crypto_resync_response(
    state: &AppState,
    user: &UserRow,
    peer_id: &str,
    message_id: &str,
    device_id: &str,
) -> Result<(), String> {
    if peer_id == user.id {
        return Err("cannot acknowledge E2E recovery from yourself".into());
    }
    if message_id.is_empty() || device_id.is_empty() {
        return Err("crypto recovery acknowledgement is missing message or device id".into());
    }

    let deleted = state
        .db
        .ack_crypto_resync_response(&user.id, peer_id, message_id, device_id)
        .map_err(|err| err.to_string())?;

    if !deleted {
        return Err("crypto recovery response is no longer pending".into());
    }

    Ok(())
}

fn relay_call_signal(
    state: &AppState,
    user: &UserRow,
    peer_id: &str,
    kind: &str,
    mut payload: serde_json::Value,
) -> Result<(), String> {
    if !matches!(
        kind,
        "offer" | "answer" | "ice_candidate" | "hangup" | "reject" | "group_invite" | "group_join"
    ) {
        return Err("unsupported call signal".into());
    }
    if peer_id == user.id {
        return Err("cannot call yourself".into());
    }

    let payload_size = serde_json::to_vec(&payload)
        .map_err(|_| "invalid call signal".to_string())?
        .len();
    if payload_size > 64 * 1024 {
        return Err("call signal is too large".into());
    }

    if let Some(group) = state.db.group(peer_id).map_err(|err| err.to_string())? {
        if !group.member_ids.iter().any(|member| member == &user.id) {
            return Err("not a member of this group".into());
        }

        let object = payload
            .as_object_mut()
            .ok_or_else(|| "group call signal payload must be an object".to_string())?;
        let target_id = object
            .get("target_id")
            .and_then(serde_json::Value::as_str)
            .filter(|id| !id.is_empty())
            .ok_or_else(|| "group call signal is missing target id".to_string())?;

        if target_id == user.id {
            return Err("cannot send a group call signal to yourself".into());
        }
        if !group.member_ids.iter().any(|member| member == target_id) {
            return Err("group call target is not a member".into());
        }

        let target = state
            .db
            .user_by_id(target_id)
            .map_err(|err| err.to_string())?
            .ok_or_else(|| "unknown group call target".to_string())?;
        if state.hub.online_ids().iter().all(|id| id != &target.id) {
            return Err("group call target is offline".into());
        }

        let recipient =
            Uuid::parse_str(&target.id).map_err(|_| "invalid group call target id".to_string())?;

        state.hub.send_to(
            recipient,
            ServerMessage::CallSignal {
                sender_id: user.id.clone(),
                kind: kind.to_string(),
                payload,
            },
        );
        return Ok(());
    }

    let peer = state
        .db
        .user_by_id(peer_id)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "unknown user".to_string())?;
    let payload_size = serde_json::to_vec(&payload)
        .map_err(|_| "invalid call signal".to_string())?
        .len();
    if payload_size > 64 * 1024 {
        return Err("call signal is too large".into());
    }
    let recipient = Uuid::parse_str(&peer.id).map_err(|_| "invalid peer id".to_string())?;
    if state.hub.online_ids().iter().all(|id| id != &peer.id) {
        return Err("user is offline".into());
    }
    state.hub.send_to(
        recipient,
        ServerMessage::CallSignal {
            sender_id: user.id.clone(),
            kind: kind.to_string(),
            payload,
        },
    );
    Ok(())
}

fn open_chat(state: &AppState, tx: &Outbound, user: &UserRow, peer_id: &str) -> Result<(), String> {
    if let Some(group) = state.db.group(peer_id).map_err(|err| err.to_string())? {
        if !group.member_ids.iter().any(|member| member == &user.id) {
            return Err("not a member of this group".into());
        }
        let history = state
            .db
            .group_history(&user.id, peer_id, HISTORY_LIMIT)
            .map_err(db_err)?;
        let online = state.hub.online_ids();
        let _ = tx.send(ServerMessage::Chat {
            peer: UserInfo {
                user_id: group.id,
                display_name: group.name,
                username: String::new(),
                email: None,
                online: group.member_ids.iter().any(|id| online.contains(id)),
                avatar_url: None,
                activity: None,
                is_group: true,
                e2e_enabled: true,
                group_member_ids: group.member_ids,
            },
            history,
        });
        return Ok(());
    }
    if peer_id == user.id {
        return Err("cannot chat with yourself".into());
    }
    let peer = state
        .db
        .user_by_id(peer_id)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "unknown user".to_string())?;
    let history = state
        .db
        .dm_history(&user.id, peer_id, HISTORY_LIMIT)
        .map_err(db_err)?;
    let online = state.hub.online_ids();
    let _ = tx.send(ServerMessage::Chat {
        peer: user_info(&peer, online.iter().any(|id| id == &peer.id)),
        history,
    });
    Ok(())
}

fn send_dm(
    state: &AppState,
    user: &UserRow,
    peer_id: &str,
    body: String,
    attachment_id: Option<String>,
    server_name: &str,
) -> Result<(), String> {
    let body = sanitize_body(&body).map_err(|err| err.to_string())?;
    if body.is_empty() && attachment_id.is_none() {
        return Err("message is empty".into());
    }
    if let Some(group) = state.db.group(peer_id).map_err(|err| err.to_string())? {
        if !group.member_ids.iter().any(|member| member == &user.id) {
            return Err("not a member of this group".into());
        }
        for member_id in &group.member_ids {
            if !state
                .db
                .user_has_crypto_devices(member_id)
                .map_err(|err| err.to_string())?
            {
                return Err("all group members must have E2E enabled".into());
            }
        }
        let attachment_is_ciphertext = match attachment_id.as_deref() {
            Some(id) => state
                .db
                .attachment(id)
                .map_err(|err| err.to_string())?
                .is_some_and(|attachment| attachment.mime == "application/octet-stream"),
            None => true,
        };
        let parsed_envelope = serde_json::from_str::<serde_json::Value>(&body).ok();
        if parsed_envelope
            .as_ref()
            .and_then(|value| value.get("version").and_then(|v| v.as_u64()))
            == Some(3)
            && parsed_envelope
                .as_ref()
                .and_then(|value| value.get("message_type").and_then(|v| v.as_str()))
                == Some("matrix")
        {
            validate_matrix_group_e2e_message(
                state,
                peer_id,
                &group.member_ids,
                &user.id,
                server_name,
                &body,
                attachment_is_ciphertext,
            )?;
        } else {
            validate_group_e2e_message(
                &body,
                &group.member_ids,
                &user.id,
                attachment_is_ciphertext,
            )?;
        }
        let message = state
            .db
            .insert_group_dm(&user.id, peer_id, &body, attachment_id.as_deref(), now_ms())
            .map_err(db_err)?;
        fanout(state, &message);
        return Ok(());
    }
    let sender_e2e = state
        .db
        .user_has_crypto_devices(&user.id)
        .map_err(|err| err.to_string())?;
    let recipient = state
        .db
        .user_by_id(peer_id)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "unknown user".to_string())?;
    let recipient_e2e = state
        .db
        .user_has_crypto_devices(&recipient.id)
        .map_err(|err| err.to_string())?;
    let attachment_is_ciphertext = match attachment_id.as_deref() {
        Some(id) => state
            .db
            .attachment(id)
            .map_err(|err| err.to_string())?
            .is_some_and(|attachment| attachment.mime == "application/octet-stream"),
        None => true,
    };
    validate_e2e_message_with_state(
        state,
        &user.id,
        peer_id,
        sender_e2e,
        recipient_e2e,
        &body,
        attachment_id.as_deref(),
        attachment_is_ciphertext,
        server_name,
    )?;
    let message = state
        .db
        .insert_dm(&user.id, peer_id, &body, attachment_id.as_deref(), now_ms())
        .map_err(db_err)?;
    fanout(state, &message);
    Ok(())
}

fn validate_matrix_group_e2e_message(
    state: &AppState,
    group_id: &str,
    member_ids: &[String],
    sender_id: &str,
    server_name: &str,
    body: &str,
    attachment_is_ciphertext: bool,
) -> Result<(), String> {
    if !attachment_is_ciphertext {
        return Err("E2E chat attachments must be encrypted before upload".into());
    }

    let envelope: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| "invalid Matrix group ciphertext envelope".to_string())?;

    if envelope.get("version").and_then(|value| value.as_u64()) != Some(3)
        || envelope
            .get("message_type")
            .and_then(|value| value.as_str())
            != Some("matrix")
    {
        return Err("invalid Matrix group ciphertext envelope".into());
    }

    let sender_device_id = envelope
        .get("sender_device_id")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Matrix group message is missing sender device id".to_string())?;

    let room_id = envelope
        .get("room_id")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Matrix group message is missing room id".to_string())?;

    let expected_room_id = format!("!larpgrp_{group_id}:{server_name}");
    if room_id != expected_room_id {
        return Err("Matrix group message has an invalid room id".into());
    }

    let ciphertext = envelope
        .get("ciphertext")
        .and_then(|value| value.as_object())
        .ok_or_else(|| "Matrix group message is missing ciphertext".to_string())?;

    if ciphertext.get("algorithm").and_then(|value| value.as_str()) != Some("m.megolm.v1.aes-sha2")
    {
        return Err("Matrix group message uses an unsupported algorithm".into());
    }

    if ciphertext
        .get("ciphertext")
        .and_then(|value| value.as_str())
        .is_none()
    {
        return Err("Matrix group message is missing ciphertext data".into());
    }

    if state
        .db
        .matrix_device(sender_id, sender_device_id)
        .map_err(|err| err.to_string())?
        .is_none()
    {
        return Err("Matrix group sender device is not registered".into());
    }

    for member_id in member_ids {
        if state
            .db
            .matrix_devices_for_user(member_id)
            .map_err(|err| err.to_string())?
            .is_empty()
        {
            return Err("all group members must have Matrix E2E devices".into());
        }
    }

    Ok(())
}

fn validate_group_e2e_message(
    body: &str,
    member_ids: &[String],
    sender_id: &str,
    attachment_is_ciphertext: bool,
) -> Result<(), String> {
    if !attachment_is_ciphertext {
        return Err("E2E chat attachments must be encrypted before upload".into());
    }
    let envelope: serde_json::Value =
        serde_json::from_str(body).map_err(|_| "invalid group ciphertext envelope".to_string())?;
    if envelope.get("version").and_then(|value| value.as_u64()) != Some(1)
        || envelope
            .get("message_type")
            .and_then(|value| value.as_str())
            != Some("group")
    {
        return Err("invalid group ciphertext envelope".into());
    }
    let ciphertexts = envelope
        .get("ciphertexts")
        .and_then(|value| value.as_object())
        .ok_or_else(|| "group message is missing member ciphertexts".to_string())?;
    for member_id in member_ids.iter().filter(|id| id.as_str() != sender_id) {
        let encrypted = ciphertexts
            .get(member_id)
            .and_then(|value| value.as_str())
            .ok_or_else(|| "group message is missing a member ciphertext".to_string())?;
        let parsed: serde_json::Value =
            serde_json::from_str(encrypted).map_err(|_| "invalid member ciphertext".to_string())?;
        if parsed.get("version").and_then(|value| value.as_u64()) != Some(1)
            || !matches!(
                parsed.get("message_type").and_then(|value| value.as_str()),
                Some("message" | "prekey")
            )
            || parsed
                .get("ciphertext")
                .and_then(|value| value.as_str())
                .is_none()
        {
            return Err("invalid member ciphertext".into());
        }
    }
    Ok(())
}

fn matrix_dm_room_id(server_name: &str, sender_user_id: &str, recipient_user_id: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut users = [sender_user_id, recipient_user_id];
    users.sort_unstable();
    let input = format!("{server_name}\n{}\n{}", users[0], users[1]);

    let digest = Sha256::digest(input.as_bytes());
    let hex = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    format!("!larpdm_{hex}:{server_name}")
}

fn validate_e2e_message_with_state(
    state: &AppState,
    sender_user_id: &str,
    recipient_user_id: &str,
    sender_e2e: bool,
    recipient_e2e: bool,
    body: &str,
    _attachment_id: Option<&str>,
    attachment_is_ciphertext: bool,
    server_name: &str,
) -> Result<(), String> {
    if !sender_e2e || !recipient_e2e {
        return Err("both participants must enable E2E before messaging".into());
    }

    if !attachment_is_ciphertext {
        return Err("E2E chat attachments must be encrypted before upload".into());
    }

    let envelope: serde_json::Value =
        serde_json::from_str(body).map_err(|_| "plaintext blocked in E2E chat".to_string())?;

    let version = envelope.get("version").and_then(|value| value.as_u64());

    let message_type = envelope
        .get("message_type")
        .and_then(|value| value.as_str());

    // Matrix-style DM envelope.
    if version == Some(3) && message_type == Some("matrix") {
        let sender_device_id = envelope
            .get("sender_device_id")
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "Matrix encrypted message is missing sender device id".to_string())?;

        let room_id = envelope
            .get("room_id")
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "Matrix encrypted message is missing room id".to_string())?;

        let expected_room_id = matrix_dm_room_id(server_name, sender_user_id, recipient_user_id);
        if room_id != expected_room_id {
            return Err("Matrix encrypted message has an invalid DM room id".into());
        }

        let ciphertext = envelope
            .get("ciphertext")
            .and_then(|value| value.as_object())
            .ok_or_else(|| "Matrix encrypted message is missing ciphertext".to_string())?;

        if ciphertext.get("algorithm").and_then(|value| value.as_str())
            != Some("m.megolm.v1.aes-sha2")
        {
            return Err("Matrix encrypted message uses an unsupported algorithm".into());
        }

        if ciphertext
            .get("ciphertext")
            .and_then(|value| value.as_str())
            .is_none()
        {
            return Err("Matrix encrypted message is missing ciphertext data".into());
        }

        if state
            .db
            .matrix_device(sender_user_id, sender_device_id)
            .map_err(|err| err.to_string())?
            .is_none()
        {
            return Err("Matrix sender device is not registered".into());
        }

        if state
            .db
            .matrix_devices_for_user(recipient_user_id)
            .map_err(|err| err.to_string())?
            .is_empty()
        {
            return Err("Matrix recipient has no registered E2E devices".into());
        }

        return Ok(());
    }

    // Multi-device DM envelope.
    if version == Some(2) && message_type == Some("message") {
        let sender_device_id = envelope
            .get("sender_device_id")
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "encrypted message is missing sender device id".to_string())?;

        let ciphertexts = envelope
            .get("ciphertexts")
            .and_then(|value| value.as_object())
            .ok_or_else(|| "encrypted message is missing device ciphertexts".to_string())?;

        if ciphertexts.is_empty() {
            return Err("encrypted message contains no device ciphertexts".into());
        }

        // The sender device must actually belong to the authenticated sender.
        let sender_devices = state
            .db
            .crypto_devices_for_user(sender_user_id)
            .map_err(|err| err.to_string())?;

        if !sender_devices
            .iter()
            .any(|device| device.device_id == sender_device_id)
        {
            return Err("sender device does not belong to authenticated sender".into());
        }

        // Every ciphertext target must actually belong to the recipient.
        let recipient_devices = state
            .db
            .crypto_devices_for_user(recipient_user_id)
            .map_err(|err| err.to_string())?;

        for (device_id, ciphertext) in ciphertexts {
            if !recipient_devices
                .iter()
                .any(|device| device.device_id == *device_id)
            {
                return Err("encrypted message contains a device not owned by recipient".into());
            }

            let ciphertext = ciphertext
                .as_str()
                .ok_or_else(|| "encrypted message contains an invalid ciphertext".to_string())?;

            // Each value is itself a normal v1 crypto envelope generated by
            // CryptoDevice::encrypt().
            let inner: serde_json::Value = serde_json::from_str(ciphertext)
                .map_err(|_| "encrypted message contains malformed ciphertext".to_string())?;

            if inner.get("version").and_then(|value| value.as_u64()) != Some(1)
                || !matches!(
                    inner.get("message_type").and_then(|value| value.as_str()),
                    Some("message" | "prekey")
                )
                || inner
                    .get("ciphertext")
                    .and_then(|value| value.as_str())
                    .is_none()
            {
                return Err("encrypted message contains invalid ciphertext envelope".into());
            }
        }

        return Ok(());
    }

    // Keep v1 DM envelopes temporarily for migration.
    if version == Some(1)
        && matches!(message_type, Some("message" | "prekey"))
        && envelope
            .get("ciphertext")
            .and_then(|value| value.as_str())
            .is_some()
    {
        return Ok(());
    }

    Err("invalid encrypted message envelope".into())
}

fn validate_e2e_message(
    sender_e2e: bool,
    recipient_e2e: bool,
    body: &str,
    _attachment_id: Option<&str>,
    attachment_is_ciphertext: bool,
) -> Result<(), String> {
    if !sender_e2e || !recipient_e2e {
        return Err("both participants must enable E2E before messaging".into());
    }

    if !attachment_is_ciphertext {
        return Err("E2E chat attachments must be encrypted before upload".into());
    }

    let envelope: serde_json::Value =
        serde_json::from_str(body).map_err(|_| "plaintext blocked in E2E chat".to_string())?;

    if envelope.get("version").and_then(|value| value.as_u64()) != Some(1)
        || !matches!(
            envelope
                .get("message_type")
                .and_then(|value| value.as_str()),
            Some("message" | "prekey")
        )
        || envelope
            .get("ciphertext")
            .and_then(|value| value.as_str())
            .is_none()
    {
        return Err("invalid encrypted message envelope".into());
    }

    Ok(())
}

fn fanout(state: &AppState, message: &ChatMessage) {
    let payload = ServerMessage::Message {
        message: message.clone(),
    };
    if let Ok(sender) = Uuid::parse_str(&message.sender_id) {
        state.hub.send_to(sender, payload.clone());
    }
    if let Ok(Some(group)) = state.db.group(&message.recipient_id) {
        for member_id in group.member_ids {
            if member_id == message.sender_id {
                continue;
            }
            if let Ok(member) = Uuid::parse_str(&member_id) {
                state.hub.send_to(member, payload.clone());
            }
        }
    } else if let Ok(recipient) = Uuid::parse_str(&message.recipient_id) {
        state.hub.send_to(recipient, payload);
    }
}

fn directory(state: &AppState, hide_email_except: &str) -> Vec<UserInfo> {
    let online = state.hub.online_ids();
    let mut users = state.db.list_users(&online).unwrap_or_default();
    for user in &mut users {
        if user.user_id == hide_email_except {
            if let Ok(Some(row)) = state.db.user_by_id(&user.user_id) {
                if !row.email.ends_with("@key.larptrix.invalid") {
                    user.email = Some(row.email);
                }
            }
        }
    }
    users
}

fn me_info(state: &AppState, user: &UserRow) -> UserInfo {
    let mut info = user_info(user, true);
    if let Ok(Some((_, username, _, _))) = state.db.profile_fields(&user.id) {
        info.username = username;
    }
    info.e2e_enabled = state.db.user_has_crypto_devices(&user.id).unwrap_or(false);
    if !user.email.ends_with("@key.larptrix.invalid") {
        info.email = Some(user.email.clone());
    }
    info
}

fn user_info(user: &UserRow, online: bool) -> UserInfo {
    UserInfo {
        user_id: user.id.clone(),
        display_name: user.display_name.clone(),
        username: String::new(),
        email: None,
        online,
        avatar_url: user.avatar_id.as_ref().map(|_| avatar_url(&user.id)),
        activity: None,
        is_group: false,
        e2e_enabled: false,
        group_member_ids: Vec::new(),
    }
}

fn db_err(err: DbError) -> String {
    match err {
        DbError::EmailTaken => "email already registered".into(),
        DbError::UsernameTaken => "username is already taken".into(),
        DbError::BadRequest(msg) => msg.into(),
        DbError::Sqlite(err) => {
            tracing::error!("db: {err}");
            "database error".into()
        }
    }
}

fn send_error(tx: &Outbound, code: &str, message: impl ToString) {
    let _ = tx.send(ServerMessage::Error {
        code: code.to_string(),
        message: message.to_string(),
    });
}

#[cfg(test)]
mod call_signal_tests {
    use super::{
        fanout, relay_call_signal, validate_e2e_message, validate_e2e_message_with_state,
        validate_group_e2e_message, validate_matrix_group_e2e_message,
    };
    use crate::db::Database;
    use crate::hub::Hub;
    use crate::AppState;
    use larptrix_protocol::{ChatMessage, ServerMessage};
    use std::path::Path;
    use tokio::sync::mpsc;
    use uuid::Uuid;

    #[test]
    fn call_signal_is_routed_only_to_online_peer() {
        let state = AppState {
            db: Database::open(Path::new(":memory:")).unwrap(),
            hub: Hub::new(),
            upload_dir: Path::new("/tmp").to_path_buf(),
        };
        let caller = state
            .db
            .create_key_user("Caller", "caller", "caller-hash", 1)
            .unwrap();
        let recipient = state
            .db
            .create_key_user("Recipient", "recipient", "recipient-hash", 1)
            .unwrap();
        let recipient_id = Uuid::parse_str(&recipient.id).unwrap();
        let (tx, mut rx) = mpsc::unbounded_channel();
        state
            .hub
            .join(recipient_id, recipient.display_name.clone(), tx);

        relay_call_signal(
            &state,
            &caller,
            &recipient.id,
            "offer",
            serde_json::json!({"type":"offer","sdp":"test-sdp"}),
        )
        .unwrap();
        let message = rx.try_recv().unwrap();
        assert!(matches!(
            message,
            ServerMessage::CallSignal { sender_id, kind, .. }
                if sender_id == caller.id && kind == "offer"
        ));
        assert!(relay_call_signal(
            &state,
            &caller,
            &recipient.id,
            "unknown",
            serde_json::json!({})
        )
        .is_err());
    }

    #[test]
    fn e2e_chats_never_accept_plaintext_or_unencrypted_attachments() {
        let envelope = r#"{"version":1,"message_type":"prekey","ciphertext":"abc"}"#;
        assert!(validate_e2e_message(false, false, "legacy plaintext", None, true).is_err());
        assert!(validate_e2e_message(true, false, envelope, None, true).is_err());
        assert!(validate_e2e_message(true, true, "legacy plaintext", None, true).is_err());
        assert!(validate_e2e_message(true, true, envelope, Some("attachment-id"), true).is_ok());
        assert!(validate_e2e_message(true, true, envelope, Some("image-id"), false).is_err());
        assert!(validate_e2e_message(true, true, envelope, None, true).is_ok());
    }

    #[test]
    fn group_messages_require_each_member_ciphertext() {
        let members = vec!["alice".to_string(), "bob".to_string(), "carol".to_string()];
        let encrypted = r#"{"version":1,"message_type":"prekey","ciphertext":"abc"}"#;
        let valid = serde_json::json!({
            "version": 1,
            "message_type": "group",
            "ciphertexts": { "bob": encrypted, "carol": encrypted }
        })
        .to_string();
        assert!(validate_group_e2e_message(&valid, &members, "alice", true).is_ok());
        let missing = serde_json::json!({
            "version": 1,
            "message_type": "group",
            "ciphertexts": { "bob": encrypted }
        })
        .to_string();
        assert!(validate_group_e2e_message(&missing, &members, "alice", true).is_err());
        assert!(validate_group_e2e_message("plaintext", &members, "alice", true).is_err());
    }

    #[test]
    fn matrix_group_message_accepts_valid_envelope_and_rejects_unknown_sender() {
        let state = AppState {
            db: Database::open(Path::new(":memory:")).unwrap(),
            hub: Hub::new(),
            upload_dir: Path::new("/tmp").to_path_buf(),
        };

        let alice = state
            .db
            .create_key_user("Alice", "alice_user", "alice", 1)
            .unwrap();
        let bob = state
            .db
            .create_key_user("Bob", "bob_user", "bob", 1)
            .unwrap();

        for (user_id, device_id) in [
            (&alice.id, "alice-matrix-device"),
            (&bob.id, "bob-matrix-device"),
        ] {
            state
                .db
                .upsert_matrix_crypto_device(
                    user_id,
                    device_id,
                    &serde_json::json!({
                        "user_id": format!("@{}:localhost", user_id),
                        "device_id": device_id,
                        "algorithms": ["m.olm.v1.curve25519-aes-sha2", "m.megolm.v1.aes-sha2"],
                        "keys": {},
                        "signatures": {}
                    })
                    .to_string(),
                    "{}",
                    "{}",
                )
                .unwrap();
        }

        let body = serde_json::json!({
            "version": 3,
            "message_type": "matrix",
            "sender_device_id": "alice-matrix-device",
            "room_id": format!("!larpgrp_group-1:localhost"),
            "ciphertext": {
                "algorithm": "m.megolm.v1.aes-sha2",
                "ciphertext": "opaque"
            }
        })
        .to_string();

        assert!(validate_matrix_group_e2e_message(
            &state,
            "group-1",
            &[alice.id.clone(), bob.id.clone()],
            &alice.id,
            "localhost",
            &body,
            true,
        )
        .is_ok());

        let fake = body.replace("alice-matrix-device", "not-alice-device");
        assert!(validate_matrix_group_e2e_message(
            &state,
            "group-1",
            &[alice.id.clone(), bob.id.clone()],
            &alice.id,
            "localhost",
            &fake,
            true,
        )
        .is_err());
    }

    #[test]
    fn multi_device_message_accepts_valid_device_ciphertexts() {
        let state = AppState {
            db: Database::open(Path::new(":memory:")).unwrap(),
            hub: Hub::new(),
            upload_dir: Path::new("/tmp").to_path_buf(),
        };

        let alice = state
            .db
            .create_key_user("Alice", "alice_user", "alice", 1)
            .unwrap();
        let bob = state
            .db
            .create_key_user("Bob", "bob_user", "bob", 1)
            .unwrap();

        state
            .db
            .create_crypto_device(
                &alice.id,
                "alice-device-1",
                r#"{"device_id":"alice-device-1"}"#,
                "encrypted-state",
            )
            .unwrap();

        state
            .db
            .create_crypto_device(
                &bob.id,
                "bob-device-1",
                r#"{"device_id":"bob-device-1"}"#,
                "encrypted-state",
            )
            .unwrap();

        let inner = r#"{"version":1,"message_type":"message","ciphertext":"abc"}"#;

        let envelope = serde_json::json!({
            "version": 2,
            "message_type": "message",
            "sender_device_id": "alice-device-1",
            "ciphertexts": {
                "bob-device-1": inner
            }
        })
        .to_string();

        assert!(validate_e2e_message_with_state(
            &state,
            &alice.id,
            &bob.id,
            true,
            true,
            &envelope,
            None,
            true,
            "localhost",
        )
        .is_ok());
    }

    #[test]
    fn multi_device_message_accepts_multiple_recipient_devices() {
        let state = AppState {
            db: Database::open(Path::new(":memory:")).unwrap(),
            hub: Hub::new(),
            upload_dir: Path::new("/tmp").to_path_buf(),
        };

        let alice = state
            .db
            .create_key_user("Alice", "alice_user", "alice", 1)
            .unwrap();
        let bob = state
            .db
            .create_key_user("Bob", "bob_user", "bob", 1)
            .unwrap();

        state
            .db
            .create_crypto_device(
                &alice.id,
                "alice-device-1",
                r#"{"device_id":"alice-device-1"}"#,
                "encrypted-state",
            )
            .unwrap();

        for device_id in ["bob-device-1", "bob-device-2"] {
            state
                .db
                .create_crypto_device(
                    &bob.id,
                    device_id,
                    &format!(r#"{{"device_id":"{device_id}"}}"#),
                    "encrypted-state",
                )
                .unwrap();
        }

        let inner = r#"{"version":1,"message_type":"message","ciphertext":"abc"}"#;

        let envelope = serde_json::json!({
            "version": 2,
            "message_type": "message",
            "sender_device_id": "alice-device-1",
            "ciphertexts": {
                "bob-device-1": inner,
                "bob-device-2": inner
            }
        })
        .to_string();

        assert!(validate_e2e_message_with_state(
            &state,
            &alice.id,
            &bob.id,
            true,
            true,
            &envelope,
            None,
            true,
            "localhost",
        )
        .is_ok());
    }

    #[test]
    fn multi_device_message_rejects_fake_sender_device() {
        let state = AppState {
            db: Database::open(Path::new(":memory:")).unwrap(),
            hub: Hub::new(),
            upload_dir: Path::new("/tmp").to_path_buf(),
        };

        let alice = state
            .db
            .create_key_user("Alice", "alice_user", "alice", 1)
            .unwrap();
        let bob = state
            .db
            .create_key_user("Bob", "bob_user", "bob", 1)
            .unwrap();

        state
            .db
            .create_crypto_device(
                &alice.id,
                "alice-device-1",
                r#"{"device_id":"alice-device-1"}"#,
                "encrypted-state",
            )
            .unwrap();

        state
            .db
            .create_crypto_device(
                &bob.id,
                "bob-device-1",
                r#"{"device_id":"bob-device-1"}"#,
                "encrypted-state",
            )
            .unwrap();

        let inner = r#"{"version":1,"message_type":"message","ciphertext":"abc"}"#;

        let envelope = serde_json::json!({
            "version": 2,
            "message_type": "message",
            "sender_device_id": "evil-device",
            "ciphertexts": {
                "bob-device-1": inner
            }
        })
        .to_string();

        assert!(validate_e2e_message_with_state(
            &state,
            &alice.id,
            &bob.id,
            true,
            true,
            &envelope,
            None,
            true,
            "localhost",
        )
        .is_err());
    }

    #[test]
    fn multi_device_message_rejects_foreign_recipient_device() {
        let state = AppState {
            db: Database::open(Path::new(":memory:")).unwrap(),
            hub: Hub::new(),
            upload_dir: Path::new("/tmp").to_path_buf(),
        };

        let alice = state
            .db
            .create_key_user("Alice", "alice_user", "alice", 1)
            .unwrap();
        let bob = state
            .db
            .create_key_user("Bob", "bob_user", "bob", 1)
            .unwrap();
        let carol = state
            .db
            .create_key_user("Carol", "carol", "carol", 1)
            .unwrap();

        state
            .db
            .create_crypto_device(
                &alice.id,
                "alice-device-1",
                r#"{"device_id":"alice-device-1"}"#,
                "encrypted-state",
            )
            .unwrap();

        state
            .db
            .create_crypto_device(
                &bob.id,
                "bob-device-1",
                r#"{"device_id":"bob-device-1"}"#,
                "encrypted-state",
            )
            .unwrap();

        state
            .db
            .create_crypto_device(
                &carol.id,
                "carol-device-1",
                r#"{"device_id":"carol-device-1"}"#,
                "encrypted-state",
            )
            .unwrap();

        let inner = r#"{"version":1,"message_type":"message","ciphertext":"abc"}"#;

        let envelope = serde_json::json!({
            "version": 2,
            "message_type": "message",
            "sender_device_id": "alice-device-1",
            "ciphertexts": {
                "carol-device-1": inner
            }
        })
        .to_string();

        assert!(validate_e2e_message_with_state(
            &state,
            &alice.id,
            &bob.id,
            true,
            true,
            &envelope,
            None,
            true,
            "localhost",
        )
        .is_err());
    }

    #[test]
    fn multi_device_message_rejects_empty_ciphertexts() {
        let state = AppState {
            db: Database::open(Path::new(":memory:")).unwrap(),
            hub: Hub::new(),
            upload_dir: Path::new("/tmp").to_path_buf(),
        };

        let alice = state
            .db
            .create_key_user("Alice", "alice_user", "alice", 1)
            .unwrap();
        let bob = state
            .db
            .create_key_user("Bob", "bob_user", "bob", 1)
            .unwrap();

        state
            .db
            .create_crypto_device(
                &alice.id,
                "alice-device-1",
                r#"{"device_id":"alice-device-1"}"#,
                "encrypted-state",
            )
            .unwrap();

        state
            .db
            .create_crypto_device(
                &bob.id,
                "bob-device-1",
                r#"{"device_id":"bob-device-1"}"#,
                "encrypted-state",
            )
            .unwrap();

        let envelope = serde_json::json!({
            "version": 2,
            "message_type": "message",
            "sender_device_id": "alice-device-1",
            "ciphertexts": {}
        })
        .to_string();

        assert!(validate_e2e_message_with_state(
            &state,
            &alice.id,
            &bob.id,
            true,
            true,
            &envelope,
            None,
            true,
            "localhost",
        )
        .is_err());
    }

    #[test]
    fn multi_device_message_rejects_malformed_inner_ciphertext() {
        let state = AppState {
            db: Database::open(Path::new(":memory:")).unwrap(),
            hub: Hub::new(),
            upload_dir: Path::new("/tmp").to_path_buf(),
        };

        let alice = state
            .db
            .create_key_user("Alice", "alice_user", "alice", 1)
            .unwrap();
        let bob = state
            .db
            .create_key_user("Bob", "bob_user", "bob", 1)
            .unwrap();

        state
            .db
            .create_crypto_device(
                &alice.id,
                "alice-device-1",
                r#"{"device_id":"alice-device-1"}"#,
                "encrypted-state",
            )
            .unwrap();

        state
            .db
            .create_crypto_device(
                &bob.id,
                "bob-device-1",
                r#"{"device_id":"bob-device-1"}"#,
                "encrypted-state",
            )
            .unwrap();

        let envelope = serde_json::json!({
            "version": 2,
            "message_type": "message",
            "sender_device_id": "alice-device-1",
            "ciphertexts": {
                "bob-device-1": "this-is-not-json"
            }
        })
        .to_string();

        assert!(validate_e2e_message_with_state(
            &state,
            &alice.id,
            &bob.id,
            true,
            true,
            &envelope,
            None,
            true,
            "localhost",
        )
        .is_err());
    }

    #[test]
    fn multi_device_message_rejects_plaintext() {
        let state = AppState {
            db: Database::open(Path::new(":memory:")).unwrap(),
            hub: Hub::new(),
            upload_dir: Path::new("/tmp").to_path_buf(),
        };

        let alice = state
            .db
            .create_key_user("Alice", "alice_user", "alice", 1)
            .unwrap();
        let bob = state
            .db
            .create_key_user("Bob", "bob_user", "bob", 1)
            .unwrap();

        state
            .db
            .create_crypto_device(
                &alice.id,
                "alice-device-1",
                r#"{"device_id":"alice-device-1"}"#,
                "encrypted-state",
            )
            .unwrap();

        state
            .db
            .create_crypto_device(
                &bob.id,
                "bob-device-1",
                r#"{"device_id":"bob-device-1"}"#,
                "encrypted-state",
            )
            .unwrap();

        assert!(validate_e2e_message_with_state(
            &state,
            &alice.id,
            &bob.id,
            true,
            true,
            "hello bob",
            None,
            true,
            "localhost",
        )
        .is_err());
    }

    #[test]
    fn group_fanout_delivers_once_to_each_member_and_not_to_outsiders() {
        let state = AppState {
            db: Database::open(Path::new(":memory:")).unwrap(),
            hub: Hub::new(),
            upload_dir: Path::new("/tmp").to_path_buf(),
        };
        let alice = state
            .db
            .create_key_user("Alice", "alice_user", "alice", 1)
            .unwrap();
        let bob = state
            .db
            .create_key_user("Bob", "bob_user", "bob", 1)
            .unwrap();
        let outsider = state
            .db
            .create_key_user("Outsider", "outsider", "outside", 1)
            .unwrap();
        let group = state
            .db
            .create_group(&alice.id, "Group", std::slice::from_ref(&bob.id))
            .unwrap();
        let (alice_tx, mut alice_rx) = mpsc::unbounded_channel();
        let (bob_tx, mut bob_rx) = mpsc::unbounded_channel();
        let (outsider_tx, mut outsider_rx) = mpsc::unbounded_channel();
        state.hub.join(
            Uuid::parse_str(&alice.id).unwrap(),
            "Alice".into(),
            alice_tx,
        );
        state
            .hub
            .join(Uuid::parse_str(&bob.id).unwrap(), "Bob".into(), bob_tx);
        state.hub.join(
            Uuid::parse_str(&outsider.id).unwrap(),
            "Outsider".into(),
            outsider_tx,
        );

        fanout(
            &state,
            &ChatMessage {
                id: "message-1".into(),
                sender_id: alice.id,
                sender_name: "Alice".into(),
                recipient_id: group.id,
                body: "ciphertext".into(),
                attachment: None,
                created_at: 1,
            },
        );

        assert!(matches!(
            alice_rx.try_recv(),
            Ok(ServerMessage::Message { .. })
        ));
        assert!(matches!(
            bob_rx.try_recv(),
            Ok(ServerMessage::Message { .. })
        ));
        assert!(alice_rx.try_recv().is_err());
        assert!(bob_rx.try_recv().is_err());
        assert!(outsider_rx.try_recv().is_err());
    }
}

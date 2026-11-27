//! Shared client application state.
//!
//! This crate is intentionally independent from the DOM and browser APIs. It can
//! be tested natively and compiled to WebAssembly for the web client.

use std::collections::BTreeMap;

use larptrix_protocol::{ChatMessage, GroupInfo, ServerMessage, UserInfo, HISTORY_LIMIT};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default)]
pub struct ClientState {
    pub current_user: Option<UserInfo>,
    pub users: BTreeMap<String, UserInfo>,
    pub groups: BTreeMap<String, GroupInfo>,
    pub chats: BTreeMap<String, Vec<ChatMessage>>,
    pub presence: BTreeMap<String, String>,
    pub active_group_calls: BTreeMap<String, GroupCallSnapshot>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroupCallSnapshot {
    pub call_id: String,
    pub media: String,
    pub initiator_id: String,
    pub participant_ids: Vec<String>,
}

#[derive(Debug, Default)]
pub struct ClientCore {
    state: ClientState,
    platform_events: Vec<ServerMessage>,
}

impl ClientCore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self) -> &ClientState {
        &self.state
    }

    /// Apply one server event to the local application state.
    ///
    /// Events that must be handled by a browser capability adapter (calls and
    /// E2E device traffic) are queued separately and can be drained by the host.
    pub fn apply_server_message(&mut self, event: ServerMessage) {
        match event {
            ServerMessage::Pong => {}
            ServerMessage::Welcome { user, users } => {
                self.upsert_user(user.clone());
                self.state.current_user = Some(user);
                for user in users {
                    self.upsert_user(user);
                }
                self.state.last_error = None;
            }
            ServerMessage::Directory { users } => {
                for user in users {
                    self.upsert_user(user);
                }
            }
            ServerMessage::Groups { groups } => {
                self.state.groups = groups
                    .into_iter()
                    .map(|group| (group.group_id.clone(), group))
                    .collect();
            }
            ServerMessage::Chat { peer, history } => {
                let peer_id = peer.user_id.clone();
                self.upsert_user(peer);
                let messages = self.state.chats.entry(peer_id).or_default();
                for message in history {
                    upsert_message(messages, message);
                }
            }
            ServerMessage::Message { message } => {
                let peer_id = self.conversation_peer_id(&message);
                let messages = self.state.chats.entry(peer_id).or_default();
                upsert_message(messages, message);
            }
            ServerMessage::MessageDeleted {
                peer_id,
                message_id,
            } => {
                if let Some(messages) = self.state.chats.get_mut(&peer_id) {
                    messages.retain(|message| message.id != message_id);
                }
            }
            ServerMessage::MessageReaction {
                peer_id,
                message_id,
                reactions,
            } => {
                if let Some(message) = self
                    .state
                    .chats
                    .get_mut(&peer_id)
                    .and_then(|messages| messages.iter_mut().find(|m| m.id == message_id))
                {
                    message.reactions = reactions;
                }
            }
            ServerMessage::MessageViewUpdate {
                peer_id,
                message_id,
                view_count,
            } => {
                if let Some(message) = self
                    .state
                    .chats
                    .get_mut(&peer_id)
                    .and_then(|messages| messages.iter_mut().find(|m| m.id == message_id))
                {
                    message.view_count = view_count;
                }
            }
            ServerMessage::Presence { user_id, status } => {
                self.state.presence.insert(user_id.clone(), status.clone());
                if let Some(user) = self.state.users.get_mut(&user_id) {
                    user.online = status != "offline";
                }
                if let Some(me) = self.state.current_user.as_mut() {
                    if me.user_id == user_id {
                        me.online = status != "offline";
                    }
                }
            }
            ServerMessage::GroupCallState {
                group_id,
                call_id,
                media,
                initiator_id,
                participant_ids,
                active,
            } => {
                if active {
                    self.state.active_group_calls.insert(
                        group_id,
                        GroupCallSnapshot {
                            call_id,
                            media,
                            initiator_id,
                            participant_ids,
                        },
                    );
                } else {
                    self.state.active_group_calls.remove(&group_id);
                }
            }
            ServerMessage::Error { code, message } => {
                self.state.last_error = Some(format!("{code}: {message}"));
            }
            other @ (ServerMessage::CallSignal { .. }
            | ServerMessage::CryptoResync { .. }
            | ServerMessage::CryptoResyncResponse { .. }
            | ServerMessage::MatrixToDevice { .. }) => {
                self.platform_events.push(other);
            }
        }
    }

    /// Drain events that must be handed to the WebRTC or E2E adapter.
    pub fn take_platform_events(&mut self) -> Vec<ServerMessage> {
        std::mem::take(&mut self.platform_events)
    }

    pub fn snapshot_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.state)
    }

    fn upsert_user(&mut self, mut user: UserInfo) {
        if let Some(status) = self.state.presence.get(&user.user_id) {
            user.online = status != "offline";
        }
        self.state.users.insert(user.user_id.clone(), user);
    }

    fn conversation_peer_id(&self, message: &ChatMessage) -> String {
        if self.state.groups.contains_key(&message.recipient_id)
            || self
                .state
                .users
                .get(&message.recipient_id)
                .is_some_and(|user| user.is_group || user.is_channel)
        {
            return message.recipient_id.clone();
        }

        match self.state.current_user.as_ref() {
            Some(me) if message.sender_id == me.user_id => message.recipient_id.clone(),
            _ => message.sender_id.clone(),
        }
    }
}

fn upsert_message(messages: &mut Vec<ChatMessage>, incoming: ChatMessage) {
    if let Some(existing) = messages
        .iter_mut()
        .find(|message| message.id == incoming.id)
    {
        *existing = incoming;
    } else {
        messages.push(incoming);
    }

    messages.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then_with(|| left.id.cmp(&right.id))
    });

    if messages.len() > HISTORY_LIMIT {
        let excess = messages.len() - HISTORY_LIMIT;
        messages.drain(0..excess);
    }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
mod wasm;

#[cfg(test)]
mod tests {
    use super::*;
    use larptrix_protocol::{AttachmentInfo, ReactionSummary};

    fn user(id: &str) -> UserInfo {
        UserInfo {
            user_id: id.to_string(),
            display_name: format!("User {id}"),
            username: id.to_string(),
            email: None,
            online: true,
            avatar_url: None,
            activity: None,
            activities: Vec::new(),
            is_group: false,
            is_channel: false,
            admin_ids: Vec::new(),
            post_policy: String::new(),
            e2e_enabled: true,
            friend_status: String::new(),
            message_policy: String::new(),
            group_member_ids: Vec::new(),
            group_description: String::new(),
            group_avatar_url: None,
            group_banner_url: None,
            subscriber_count: 0,
            tags: Vec::new(),
        }
    }

    fn message(id: &str, sender: &str, recipient: &str, created_at: i64) -> ChatMessage {
        ChatMessage {
            id: id.to_string(),
            sender_id: sender.to_string(),
            sender_name: format!("User {sender}"),
            recipient_id: recipient.to_string(),
            body: format!("message {id}"),
            attachment: None,
            attachments: Vec::<AttachmentInfo>::new(),
            reactions: Vec::<ReactionSummary>::new(),
            view_count: 0,
            created_at,
        }
    }

    fn signed_in_core() -> ClientCore {
        let mut core = ClientCore::new();
        core.apply_server_message(ServerMessage::Welcome {
            user: user("alice"),
            users: vec![user("bob")],
        });
        core
    }

    #[test]
    fn merges_history_deduplicates_and_sorts_messages() {
        let mut core = signed_in_core();
        core.apply_server_message(ServerMessage::Chat {
            peer: user("bob"),
            history: vec![
                message("second", "bob", "alice", 20),
                message("first", "alice", "bob", 10),
            ],
        });
        core.apply_server_message(ServerMessage::Message {
            message: message("second", "bob", "alice", 20),
        });

        let chat = &core.state().chats["bob"];
        assert_eq!(chat.len(), 2);
        assert_eq!(chat[0].id, "first");
        assert_eq!(chat[1].id, "second");
    }

    #[test]
    fn deletion_reaction_and_view_updates_target_the_correct_message() {
        let mut core = signed_in_core();
        core.apply_server_message(ServerMessage::Chat {
            peer: user("bob"),
            history: vec![message("m1", "bob", "alice", 1)],
        });
        core.apply_server_message(ServerMessage::MessageReaction {
            peer_id: "bob".into(),
            message_id: "m1".into(),
            reactions: vec![ReactionSummary {
                emoji: "🔥".into(),
                count: 2,
                reacted: true,
            }],
        });
        core.apply_server_message(ServerMessage::MessageViewUpdate {
            peer_id: "bob".into(),
            message_id: "m1".into(),
            view_count: 7,
        });
        assert_eq!(core.state().chats["bob"][0].view_count, 7);
        assert_eq!(core.state().chats["bob"][0].reactions[0].count, 2);

        core.apply_server_message(ServerMessage::MessageDeleted {
            peer_id: "bob".into(),
            message_id: "m1".into(),
        });
        assert!(core.state().chats["bob"].is_empty());
    }

    #[test]
    fn presence_is_preserved_when_a_user_is_upserted_later() {
        let mut core = signed_in_core();
        core.apply_server_message(ServerMessage::Presence {
            user_id: "bob".into(),
            status: "offline".into(),
        });
        core.apply_server_message(ServerMessage::Directory {
            users: vec![user("bob")],
        });
        assert!(!core.state().users["bob"].online);
    }

    #[test]
    fn incoming_and_outgoing_group_messages_share_the_group_conversation() {
        let mut core = signed_in_core();
        core.apply_server_message(ServerMessage::Groups {
            groups: vec![GroupInfo {
                group_id: "group-1".into(),
                name: "Friends".into(),
                member_ids: vec!["alice".into(), "bob".into()],
                is_channel: false,
                admin_ids: vec!["alice".into()],
                post_policy: "members".into(),
                description: String::new(),
                avatar_url: None,
                banner_url: None,
                subscriber_count: 2,
            }],
        });

        core.apply_server_message(ServerMessage::Message {
            message: message("from-alice", "alice", "group-1", 1),
        });
        core.apply_server_message(ServerMessage::Message {
            message: message("from-bob", "bob", "group-1", 2),
        });

        let group_chat = &core.state().chats["group-1"];
        assert_eq!(group_chat.len(), 2);
        assert_eq!(group_chat[0].id, "from-alice");
        assert_eq!(group_chat[1].id, "from-bob");
        assert!(!core.state().chats.contains_key("bob"));
    }

    #[test]
    fn group_call_state_is_added_and_removed() {
        let mut core = signed_in_core();
        core.apply_server_message(ServerMessage::GroupCallState {
            group_id: "group-1".into(),
            call_id: "call-1".into(),
            media: "video".into(),
            initiator_id: "alice".into(),
            participant_ids: vec!["alice".into()],
            active: true,
        });
        assert!(core.state().active_group_calls.contains_key("group-1"));
        core.apply_server_message(ServerMessage::GroupCallState {
            group_id: "group-1".into(),
            call_id: "call-1".into(),
            media: "video".into(),
            initiator_id: "alice".into(),
            participant_ids: Vec::new(),
            active: false,
        });
        assert!(!core.state().active_group_calls.contains_key("group-1"));
    }
}

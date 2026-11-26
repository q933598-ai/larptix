use async_channel::Sender;
use gpui::{
    div, prelude::*, rgb, px, App, Context, FocusHandle, KeyDownEvent, MouseButton, Window,
};
use larptrix_client_core::ClientCore;
use larptrix_protocol::{ClientMessage, ServerMessage};
use tokio::runtime::Handle;

use crate::network::{self, AppEvent};

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
}

pub struct LarptrixShell {
    runtime: Handle,
    event_tx: Sender<AppEvent>,
    core: ClientCore,
    outgoing: Option<Sender<ClientMessage>>,
    server_url: String,
    access_key: String,
    input_focus: FocusHandle,
    active_input: Option<LoginField>,
    connecting: bool,
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
            server_url: std::env::var("LARPTRIX_SERVER_URL").unwrap_or_default(),
            access_key: String::new(),
            input_focus: cx.focus_handle(),
            active_input: Some(LoginField::Server),
            connecting: false,
            connection_status: "Enter your server address and access key".into(),
            active_section: Section::Messages,
            selected_peer: None,
        }
    }

    fn handle_event(&mut self, event: AppEvent, cx: &mut Context<Self>) {
        match event {
            AppEvent::Authenticated { user, outgoing } => {
                self.connecting = false;
                self.access_key.clear();
                self.outgoing = Some(outgoing);
                self.core.apply_server_message(ServerMessage::Welcome {
                    user,
                    users: Vec::new(),
                });
                self.connection_status = "Signed in · synchronizing".into();
                self.active_section = Section::Messages;
            }
            AppEvent::Server(message) => {
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

                if self.selected_peer.is_none() {
                    if let Some(peer_id) = suggested_peer {
                        self.open_chat(peer_id);
                    }
                }

                if self.core.state().current_user.is_some() {
                    self.connection_status = "Connected".into();
                }
            }
            AppEvent::Status(status) => {
                self.connection_status = status;
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
        let access_key = self.access_key.trim().to_string();

        if server_url.is_empty() {
            self.connection_status = "Enter your Larptrix server URL".into();
            cx.notify();
            return;
        }
        if access_key.len() != 64 || !access_key.chars().all(|ch| ch.is_ascii_hexdigit()) {
            self.connection_status = "The access key must contain 64 hexadecimal characters".into();
            cx.notify();
            return;
        }

        self.connecting = true;
        self.connection_status = "Starting secure session…".into();
        cx.notify();

        let events = self.event_tx.clone();
        self.runtime.spawn(async move {
            if let Err(error) = network::connect(server_url, access_key, events.clone()).await {
                let _ = events.send(AppEvent::Error(error)).await;
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

        if event.keystroke.key == "backspace" {
            match field {
                LoginField::Server => {
                    self.server_url.pop();
                }
                LoginField::AccessKey => {
                    self.access_key.pop();
                }
            }
            cx.notify();
            return;
        }

        if event.keystroke.key == "enter" {
            self.start_connection(cx);
            return;
        }

        let is_paste = event.keystroke.key == "v"
            && (event.keystroke.modifiers.control || event.keystroke.modifiers.platform);
        if is_paste {
            if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                let current_len = match field {
                    LoginField::Server => self.server_url.len(),
                    LoginField::AccessKey => self.access_key.len(),
                };
                let max_len = match field {
                    LoginField::Server => 256usize,
                    LoginField::AccessKey => 64usize,
                };
                let remaining = max_len.saturating_sub(current_len);
                let pasted: String = text
                    .chars()
                    .filter(|ch| !ch.is_control())
                    .filter(|ch| field != LoginField::AccessKey || ch.is_ascii_hexdigit())
                    .take(remaining)
                    .collect();
                match field {
                    LoginField::Server => self.server_url.push_str(&pasted),
                    LoginField::AccessKey => self.access_key.push_str(&pasted),
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
                LoginField::AccessKey => (self.access_key.len(), 64usize),
            };
            if current_len + text.len() > max_len {
                return;
            }
            if field == LoginField::AccessKey
                && !text.chars().all(|ch| ch.is_ascii_hexdigit())
            {
                return;
            }
            if !text.chars().any(char::is_control) {
                match field {
                    LoginField::Server => self.server_url.push_str(text),
                    LoginField::AccessKey => self.access_key.push_str(text),
                }
                cx.notify();
            }
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
        let display_value = if field == LoginField::AccessKey && !value.is_empty() {
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

    fn open_chat(&mut self, peer_id: String) {
        self.selected_peer = Some(peer_id.clone());
        if let Some(outgoing) = &self.outgoing {
            let _ = outgoing.try_send(ClientMessage::Open { peer_id });
        }
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
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.active_section = Section::Messages;
                    cx.notify();
                })),
            )
            .child(
                rail_button(
                    "rail-friends",
                    "♙",
                    self.active_section == Section::Friends,
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.active_section = Section::Friends;
                    cx.notify();
                })),
            )
            .child(
                rail_button(
                    "rail-groups",
                    "▦",
                    self.active_section == Section::Groups,
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.active_section = Section::Groups;
                    cx.notify();
                })),
            )
            .child(div().flex_1())
            .child(
                rail_button(
                    "rail-settings",
                    "⚙",
                    self.active_section == Section::Settings,
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.active_section = Section::Settings;
                    cx.notify();
                })),
            )
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match self.active_section {
            Section::Messages => self.message_list(cx).into_any_element(),
            Section::Friends => self.friends_list().into_any_element(),
            Section::Groups => self.groups_list().into_any_element(),
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
            .when(self.core.state().users.is_empty(), |element| {
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

    fn groups_list(&self) -> impl IntoElement {
        let groups: Vec<_> = self
            .core
            .state()
            .groups
            .values()
            .map(|group| {
                (
                    group.name.clone(),
                    group.member_ids.len(),
                    group.is_channel,
                    group.description.clone(),
                )
            })
            .collect();

        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .children(groups.iter().map(|(name, members, is_channel, description)| {
                group_row(
                    if *is_channel { "▤" } else { "⌘" }.into(),
                    name.clone(),
                    format!("{members} members"),
                    description.clone(),
                )
            }))
            .when(groups.is_empty(), |element| {
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
            .child(settings_row("Account", "Account profile and active sessions"))
            .child(settings_row(
                "End-to-end encryption",
                "Native device crypto integration is the next milestone",
            ))
            .child(settings_row(
                "Appearance",
                "Native theme and layout preferences",
            ))
    }

    fn conversation(&self) -> impl IntoElement {
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
                let looks_encrypted = serde_json::from_str::<serde_json::Value>(&message.body).is_ok();
                let body = if looks_encrypted {
                    "Encrypted message · waiting for native E2E decryption".to_string()
                } else {
                    message.body.clone()
                };
                (author, body, mine)
            })
            .collect();

        let display_name = peer.display_name.clone();
        let handle = if peer.username.is_empty() {
            String::new()
        } else {
            format!("@{}", peer.username)
        };
        let status = if peer.online { "Online" } else { "Offline" };

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
                            .child(handle),
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
                    .children(messages.iter().map(|(author, body, mine)| {
                        message_bubble(author.clone(), body.clone(), *mine)
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
                            .text_sm()
                            .text_color(rgb(MUTED))
                            .child("＋"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .text_color(rgb(MUTED))
                            .child("Message composer will unlock after native E2E is connected"),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(SURFACE))
                            .text_sm()
                            .text_color(rgb(MUTED))
                            .child("E2E"),
                    ),
            )
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
                .child(self.server_rail(cx))
                .child(self.sidebar(cx))
                .child(self.conversation())
                .into_any_element()
        }
    }
}

impl LarptrixShell {
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
                    .child(
                        div()
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
                            .child("Your access key is used only to create a session. It is not saved to disk."),
                    ),
            )
    }
}

fn users_is_empty(total_users: usize) -> bool {
    total_users <= 1
}

fn rail_button(id: &'static str, icon: &'static str, selected: bool) -> impl IntoElement {
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

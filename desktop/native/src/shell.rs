use gpui::{
    div, prelude::*, rgb, px, App, Context, Window,
};
use larptrix_client_core::ClientCore;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Section {
    #[default]
    Messages,
    Friends,
    Groups,
    Settings,
}

impl Section {
    fn eyebrow(self) -> &'static str {
        match self {
            Self::Messages => "DIRECT MESSAGES",
            Self::Friends => "FRIENDS",
            Self::Groups => "YOUR GROUPS",
            Self::Settings => "PREFERENCES",
        }
    }
}

pub struct LarptrixShell {
    active_section: Section,
    selected_chat: &'static str,
    _core: ClientCore,
}

impl Default for LarptrixShell {
    fn default() -> Self {
        Self {
            active_section: Section::Messages,
            selected_chat: "mira",
            _core: ClientCore::new(),
        }
    }
}

impl LarptrixShell {
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
                div()
                    .id("rail-messages")
                    .size(px(44.0))
                    .rounded_xl()
                    .bg(if self.active_section == Section::Messages {
                        rgb(SURFACE_HOVER)
                    } else {
                        rgb(SURFACE)
                    })
                    .text_color(rgb(TEXT))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .child("◈")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.active_section = Section::Messages;
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .id("rail-friends")
                    .size(px(44.0))
                    .rounded_xl()
                    .bg(if self.active_section == Section::Friends {
                        rgb(SURFACE_HOVER)
                    } else {
                        rgb(SURFACE)
                    })
                    .text_color(rgb(TEXT))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .child("♙")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.active_section = Section::Friends;
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .id("rail-groups")
                    .size(px(44.0))
                    .rounded_xl()
                    .bg(if self.active_section == Section::Groups {
                        rgb(SURFACE_HOVER)
                    } else {
                        rgb(SURFACE)
                    })
                    .text_color(rgb(TEXT))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .child("▦")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.active_section = Section::Groups;
                        cx.notify();
                    })),
            )
            .child(div().flex_1())
            .child(
                div()
                    .id("rail-settings")
                    .size(px(44.0))
                    .rounded_xl()
                    .bg(if self.active_section == Section::Settings {
                        rgb(SURFACE_HOVER)
                    } else {
                        rgb(SURFACE)
                    })
                    .text_color(rgb(MUTED))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .child("⚙")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.active_section = Section::Settings;
                        cx.notify();
                    })),
            )
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let title = match self.active_section {
            Section::Messages => "Messages",
            Section::Friends => "Friends",
            Section::Groups => "Groups",
            Section::Settings => "Settings",
        };

        let content = match self.active_section {
            Section::Messages => div().flex_1().child(self.message_list(cx)),
            Section::Friends => div().flex_1().child(self.friends_list()),
            Section::Groups => div().flex_1().child(self.groups_list()),
            Section::Settings => div().flex_1().child(self.settings_list()),
        };

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
                            .child(title),
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
                            .child("Y"),
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
                                    .child("Your account"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(MUTED))
                                    .child("Local preview"),
                            ),
                    )
                    .child(div().size(px(7.0)).rounded_full().bg(rgb(GREEN))),
            )
    }

    fn message_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_1()
            .px_2()
            .child(chat_entry("mira", "Mira", "Are you still working on it?", "2m", self.selected_chat == "mira", cx.listener(|this, _, _, cx| {
                this.selected_chat = "mira";
                cx.notify();
            })))
            .child(chat_entry("alex", "Alex", "that actually looks great", "18m", self.selected_chat == "alex", cx.listener(|this, _, _, cx| {
                this.selected_chat = "alex";
                cx.notify();
            })))
            .child(chat_entry("sam", "Sam", "sent an attachment", "1h", self.selected_chat == "sam", cx.listener(|this, _, _, cx| {
                this.selected_chat = "sam";
                cx.notify();
            })))
            .child(chat_entry("dev", "Dev corner", "Pinned: project roadmap", "3h", self.selected_chat == "dev", cx.listener(|this, _, _, cx| {
                this.selected_chat = "dev";
                cx.notify();
            })))
    }

    fn friends_list(&self) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .child(section_heading("ONLINE"))
            .child(person_row("M", "Mira", "Online"))
            .child(person_row("A", "Alex", "Online"))
            .child(section_heading("OFFLINE"))
            .child(person_row("S", "Sam", "Last seen recently"))
    }

    fn groups_list(&self) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .child(group_row("⌘", "Dev corner", "4 members"))
            .child(group_row("◉", "Game nights", "6 members"))
            .child(group_row("◇", "Project notes", "3 members"))
    }

    fn settings_list(&self) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .child(settings_row("Account", "Profile and devices"))
            .child(settings_row("Appearance", "Themes and layout"))
            .child(settings_row("Voice & video", "Audio and camera"))
            .child(settings_row("Privacy", "Sessions and encryption"))
            .child(settings_row("Notifications", "Sounds and alerts"))
    }

    fn conversation(&self) -> impl IntoElement {
        let (name, handle, initials, topic) = match self.selected_chat {
            "alex" => ("Alex", "@alex", "A", "Usually replies quickly"),
            "sam" => ("Sam", "@sam", "S", "Away"),
            "dev" => ("Dev corner", "#dev-corner", "D", "A small space for building things"),
            _ => ("Mira", "@mira", "M", "Online"),
        };

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
                            .child(initials),
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
                                    .child(name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(MUTED))
                                    .child(topic),
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
                    .child(message_bubble("Mira", "Hey! How is the new Larptrix build going?", false))
                    .child(message_bubble("You", "I'm rebuilding the desktop client as a native Rust app.", true))
                    .child(message_bubble(name, "No Electron? That should feel much lighter.", false))
                    .child(message_bubble("You", "Exactly. Shared Rust core, separate native and web interfaces.", true))
                    .child(
                        div()
                            .pt_2()
                            .text_xs()
                            .text_color(rgb(MUTED))
                            .flex()
                            .justify_center()
                            .child("Today · local interface preview"),
                    ),
            )
            .child(
                div()
                    .mx_5()
                    .mb_4()
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
                            .text_color(rgb(MUTED))
                            .text_sm()
                            .child("＋"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .text_color(rgb(MUTED))
                            .child(format!("Message {handle}…")),
                    )
                    .child(
                        div()
                            .text_color(rgb(MUTED))
                            .text_sm()
                            .child("☺"),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(ACCENT))
                            .text_color(rgb(RAIL))
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Send"),
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
                    .child("Native GPUI preview · not connected to a server"),
            )
    }
}

impl Render for LarptrixShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_row()
            .bg(rgb(BG))
            .text_color(rgb(TEXT))
            .child(self.server_rail(cx))
            .child(self.sidebar(cx))
            .child(self.conversation())
    }
}

fn chat_entry(
    id: &'static str,
    name: &'static str,
    preview: &'static str,
    time: &'static str,
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
        .bg(if selected { rgb(SURFACE_HOVER) } else { rgb(SIDEBAR) })
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

fn message_bubble(author: &'static str, body: &'static str, mine: bool) -> impl IntoElement {
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
                .bg(if mine { rgb(0x35495b) } else { rgb(SURFACE) })
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

fn person_row(initial: &'static str, name: &'static str, status: &'static str) -> impl IntoElement {
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

fn group_row(icon: &'static str, name: &'static str, members: &'static str) -> impl IntoElement {
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
                .child(div().text_xs().text_color(rgb(MUTED)).child(members)),
        )
}

fn settings_row(title: &'static str, subtitle: &'static str) -> impl IntoElement {
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

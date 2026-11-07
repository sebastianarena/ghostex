//! Your messages under the opt-in chat look (`chatLook` in the chat document): Message layout's
//! Left aligned and the Band, Left rule and Names styles of Your messages. At Default nothing here
//! runs and your message is today's right-aligned bubble (transcript.rs `message_row`).

use super::{appearance::ChatAppearance, state::NativeChatView, transcript::text};
use ghostex_gx_chat_core::transcript::chat_look::{MessageLayout, UserStyle};
use gpui::prelude::FluentBuilder as _;
use gpui::{
    AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div, px, relative, svg,
};
use serde_json::Value;

/// The avatar square opening a Names line, before the zoom.
const AVATAR: f32 = 18.0;
/// How far a Names row's content sits in from its name line: the avatar and the 8px gap after it.
pub(super) const NAMES_INDENT: f32 = 26.0;

impl ChatAppearance {
    /// Whether this chat draws your messages on the left (Message layout, or any Your messages
    /// style, which the core already folded into `messageLayout`).
    pub(crate) fn left_aligned(&self) -> bool {
        self.chat_look.effective_layout() == MessageLayout::Left
    }

    /// Whether Your messages is Names: a name line opens your messages and the agent's replies.
    pub(crate) fn names(&self) -> bool {
        self.chat_look.user_style == UserStyle::Names
    }
}

impl NativeChatView {
    /// Your messages and Message layout on your rows. Left aligned keeps the bubble and moves it to the agent's left edge; Band draws your message across the column on a fill one step off the bubble's, Left rule as brighter text after a bar in the link colour, and Names opens it with an avatar, "You" and the time, the text indented, while the agent's reply opens with its own name and logo. Each style applies to every row you wrote (sent, queued, waiting or not sent); the delivery and queued icons sit at the message's right end and the hover actions under it on the left.
    ///
    /// CDXC:SessionChat 2026-11-07 WHY:
    /// The sizes are the builder's pick from the mockups, not the user's: the 3px Left rule bar and the 26px Names indent (the avatar's width plus its gap, so the text lines up under "You").
    ///
    /// CDXC:SessionChat 2026-11-07 SEE-ALSO:
    /// packages/gx-chat-core/src/transcript/chat_look.rs (the user decisions, the settings and the implied Left), apps/mobile/app/src/chat/native/transcript/userMessageLook.tsx (the phone draws the same looks).
    pub(super) fn left_user_message(
        &mut self,
        row: gpui::Stateful<gpui::Div>,
        id: &str,
        message: &Value,
        body: String,
        p: &ChatAppearance,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let s = p.scale;
        let style = p.chat_look.user_style;
        let mut text_look = p.clone();
        text_look.prose = match style {
            UserStyle::Default => p.strengthened(p.primary),
            UserStyle::Band | UserStyle::Rule | UserStyle::Names => p.user_style_prose(),
        };
        text_look.code_text = p.primary;
        let indicator = self.delivery_indicator(message, p, cx);
        let prompt = div()
            .min_w_0()
            // A slash command reads as one, in the chat's monospace (gx-chat-core `is_slash_command_line`).
            .when(message["slashCommand"] == true, |prompt| {
                prompt.font_family(super::fonts::CHAT_MONO)
            })
            .child(self.prompt_body(id, message, body, &text_look, p, cx));
        let frame = match style {
            UserStyle::Default => {
                // The bubble's indicator hangs off its right edge, level with its last line, at the
                // same 6px gap the right-aligned bubble keeps on its left.
                let indicator = indicator.map(|(icon, slots)| {
                    let offset = 31.0 + 26.0 * slots.saturating_sub(1) as f32;
                    div()
                        .absolute()
                        .bottom(px(11.0 * s))
                        .right(px(-offset * s))
                        .child(icon)
                });
                div()
                    .relative()
                    .max_w(relative(0.8))
                    .min_w_0()
                    .rounded(px(super::chat_text_look::USER_BUBBLE_RADIUS * s))
                    .border(px(1.0 * s))
                    .border_color(gpui::transparent_black())
                    .p(px(super::chat_text_look::USER_BUBBLE_PADDING * s))
                    .bg(p.input)
                    .child(prompt)
                    .children(indicator)
            }
            UserStyle::Band | UserStyle::Rule | UserStyle::Names => {
                // Full width, so the icons take the row's right end inside it.
                let line = div()
                    .w_full()
                    .min_w_0()
                    .flex()
                    .items_end()
                    .gap(px(8.0 * s))
                    .child(prompt.flex_1())
                    .children(indicator.map(|(icon, _)| icon));
                match style {
                    UserStyle::Band => line
                        .rounded(px(8.0 * s))
                        .px(px(12.0 * s))
                        .py(px(8.0 * s))
                        .bg(p.user_band),
                    UserStyle::Rule => line
                        .border_l(px(3.0 * s))
                        .border_color(
                            super::markdown_links::transcript_link_color(p).unwrap_or(p.primary),
                        )
                        .pl(px(12.0 * s)),
                    _ => line,
                }
            }
        };
        let failed_delivery = self.render_startup_delivery(message, false, p, cx);
        let thumbnails = self.user_image_thumbnails(message, p, cx);
        let actions = self.user_actions(message, p, cx);
        // A subagent's transcript draws no name lines, for your rows as for the agent's
        // (transcript.rs).
        let names = style == UserStyle::Names && !self.in_subagent;
        let indent = |element: gpui::Div| {
            if names {
                element.pl(px(NAMES_INDENT * s))
            } else {
                element
            }
        };
        let thumbnails = thumbnails.map(|images| indent(div().w_full().child(images)));
        let column = indent(
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(4.0 * s))
                .child(frame)
                .children(failed_delivery)
                .children(actions),
        );
        if !names {
            return row.children(thumbnails).child(column).into_any_element();
        }
        // The text follows its name line directly, as the agent's reply follows its own.
        let you = name_line(
            avatar(
                svg()
                    .path("titlebar/user.svg")
                    .size(px(12.0 * s))
                    .text_color(p.background)
                    .into_any_element(),
                super::markdown_links::transcript_link_color(p).unwrap_or(p.primary),
                p,
            ),
            "You".into(),
            speaker_time(id, message, p),
            p,
        );
        row.child(
            div().w_full().flex().flex_col().child(you).child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap(px(super::transcript::MESSAGE_PART_GAP * s))
                    .children(thumbnails)
                    .child(column),
            ),
        )
        .into_any_element()
    }

    /// The name line that opens the agent's reply under Names: its logo on its accent, then its
    /// name, both from the document's `chatSpeaker` (gx-chat-core `chat_look::chat_speaker`, which
    /// the phone reads too). Nothing when the core names no speaker.
    pub(super) fn agent_name_line(&self, p: &ChatAppearance) -> Option<AnyElement> {
        let s = p.scale;
        let speaker = &self.snapshot["chatSpeaker"];
        let name = speaker["name"]
            .as_str()
            .filter(|name| !name.is_empty())?
            .to_owned();
        let icon = speaker["icon"].as_str().map(str::to_owned);
        let glyph = match &icon {
            Some(icon) => svg()
                .path(format!("agent-icons/{icon}.svg"))
                .size(px(12.0 * s))
                .text_color(p.background)
                .into_any_element(),
            None => div()
                .text_size(px(10.0 * s))
                .text_color(p.background)
                .child(name.chars().take(1).collect::<String>())
                .into_any_element(),
        };
        let fill = icon.as_deref().map_or(p.muted, |icon| {
            gpui::rgb(super::option_pills::agent_accent(icon, p)).into()
        });
        Some(name_line(avatar(glyph, fill, p), name.into(), None, p))
    }
}

/// An 18px rounded square on `fill` holding `glyph`.
fn avatar(glyph: AnyElement, fill: gpui::Hsla, p: &ChatAppearance) -> gpui::Div {
    let s = p.scale;
    div()
        .size(px(AVATAR * s))
        .flex_shrink_0()
        .rounded(px(5.0 * s))
        .bg(fill)
        .flex()
        .items_center()
        .justify_center()
        .font_weight(gpui::FontWeight::BOLD)
        .child(glyph)
}

/// A Names line: the avatar, the speaker in bold, then the time when there is one. Plain text
/// children, so selecting or copying a message never picks it up.
fn name_line(
    avatar: gpui::Div,
    speaker: gpui::SharedString,
    time: Option<AnyElement>,
    p: &ChatAppearance,
) -> AnyElement {
    let s = p.scale;
    div()
        .flex()
        .items_center()
        .gap(px(8.0 * s))
        .text_size(px(12.5 * s))
        .line_height(px(20.0 * s))
        .font_weight(gpui::FontWeight::BOLD)
        .text_color(p.foreground)
        .child(avatar)
        .child(speaker)
        .children(time)
        .into_any_element()
}

/// The time on your Names line (gx-chat-core transcript/message_time.rs), with its long form as
/// the tooltip; the hover actions then leave it out, so it shows once.
fn speaker_time(id: &str, message: &Value, p: &ChatAppearance) -> Option<AnyElement> {
    let label = text(&message["time"], "label");
    if label.is_empty() {
        return None;
    }
    let title = gpui::SharedString::from(text(&message["time"], "title"));
    Some(
        div()
            .id(gpui::SharedString::from(format!("message-name-time:{id}")))
            .flex_shrink_0()
            .whitespace_nowrap()
            .text_size(px(11.5 * p.scale))
            .font_weight(gpui::FontWeight::NORMAL)
            .text_color(p.muted)
            .tooltip(move |window, cx| {
                gpui_component::tooltip::Tooltip::new(title.clone()).build(window, cx)
            })
            .child(label)
            .into_any_element(),
    )
}

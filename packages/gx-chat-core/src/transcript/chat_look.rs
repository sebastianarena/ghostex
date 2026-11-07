//! The chat look: four opt-in Settings > Chat rows (Message layout, Your messages, Agent work, Status line in the chat) that change how the transcript reads. With all four at Default the chat is today's, and nothing here runs.
//!
//! The settings ride [`crate::ChatSettings::chat_look`] from the desktop host and the chat read (`chatLook`) from gxserver, so the desktop, the web build and the phone draw the same look. The document carries [`ChatLook`] as `chatLook`, absent while every setting is Default.
//!
//! CDXC:SessionChat 2026-11-07 DECISION:
//! Message layout (`chatMessageLayout`). User: "align all chats to the left like Slack or Claude Code". Left aligned starts every message at the same left edge. It is opt-in: maddada's rule as Sebastian relayed it, "all of these things need to be advanced mode and if we don't enable anything then it should remain to look like it does now".
//!
//! CDXC:SessionChat 2026-11-07 DECISION:
//! Your messages (`chatUserMessageStyle`). User: "what needs to shine is the text". Band, Left rule and Names replace the bubble, and any of them also left-aligns the chat whatever Message layout says ([`ChatLook::effective_layout`]). Default keeps the bubble on the right: "all of these things need to be advanced mode and if we don't enable anything then it should remain to look like it does now".
//!
//! CDXC:SessionChat 2026-11-07 DECISION:
//! Agent work (`chatAgentWorkStyle`). User: the agent's work should be "way less prominent, the opposite of a panel, with an icon and a checkmark". Quiet lines and Grouped turn the tool call cards off in the projection whatever Tool call cards says (`TranscriptViewState::effective_tool_cards`). Approvals, questions, sent-message cards, Workflow cards, artifacts and the read-image strip never fold into a quiet or grouped line. Default keeps today's Worked for card: "all of these things need to be advanced mode and if we don't enable anything then it should remain to look like it does now".
//!
//! CDXC:SessionChat 2026-11-07 DECISION:
//! Status line in the chat (`chatStatusInTranscript`). User: "the line that has the status that shows at the top of the input bar, that I really enjoyed in Claude to be part of the messages". When it is on, a finished turn's fold keeps its own "Worked for" time and the card above the chat box never shows the worked line (`transcript::rows`, `extras::working_card`). Off by default: "all of these things need to be advanced mode and if we don't enable anything then it should remain to look like it does now".
//!
//! CDXC:SessionChat 2026-11-07 SEE-ALSO:
//! packages/settings-catalog/src/modules.rs (the rows), server/src/session_chat_read.rs (`chatLook` on every read), apps/desktop/src/app/gx_chat/boot.rs and world.rs (the desktop host's read and live updates), apps/desktop/src/app/native_chat/ (desktop and web) and apps/mobile/app/src/chat/ (phone) draw the document's `chatLook` and must match.

use serde::{Deserialize, Serialize};

pub use ghostex_gx_protocol::ChatLookSettings;

/// Message layout (`chatMessageLayout`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageLayout {
    #[default]
    Default,
    Left,
}

/// Your messages (`chatUserMessageStyle`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStyle {
    #[default]
    Default,
    Band,
    Rule,
    Names,
}

/// Agent work (`chatAgentWorkStyle`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentWork {
    #[default]
    Default,
    Quiet,
    Grouped,
}

/// The four chat look settings the core projects and publishes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatLook {
    pub message_layout: MessageLayout,
    pub user_style: UserStyle,
    pub agent_work: AgentWork,
    /// Status line in the chat (`chatStatusInTranscript`).
    pub status_in_transcript: bool,
}

/// A stored Message layout; an unknown value reads as Default.
pub fn parse_message_layout(value: &str) -> MessageLayout {
    match value {
        "left" => MessageLayout::Left,
        _ => MessageLayout::Default,
    }
}

/// A stored Your messages style; an unknown value reads as Default.
pub fn parse_user_style(value: &str) -> UserStyle {
    match value {
        "band" => UserStyle::Band,
        "rule" => UserStyle::Rule,
        "names" => UserStyle::Names,
        _ => UserStyle::Default,
    }
}

/// A stored Agent work style; an unknown value reads as Default.
pub fn parse_agent_work(value: &str) -> AgentWork {
    match value {
        "quiet" => AgentWork::Quiet,
        "grouped" => AgentWork::Grouped,
        _ => AgentWork::Default,
    }
}

impl ChatLook {
    /// Every setting is at Default: today's chat.
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    /// The layout the chat draws: Left whenever Your messages is not Default.
    pub fn effective_layout(&self) -> MessageLayout {
        if self.user_style != UserStyle::Default {
            MessageLayout::Left
        } else {
            self.message_layout
        }
    }

    /// This look with what `settings` names; a field it leaves out keeps its value.
    pub fn with_settings(&self, settings: &ChatLookSettings) -> Self {
        Self {
            message_layout: settings
                .message_layout
                .as_deref()
                .map_or(self.message_layout, parse_message_layout),
            user_style: settings
                .user_style
                .as_deref()
                .map_or(self.user_style, parse_user_style),
            agent_work: settings
                .agent_work
                .as_deref()
                .map_or(self.agent_work, parse_agent_work),
            status_in_transcript: settings
                .status_in_transcript
                .unwrap_or(self.status_in_transcript),
        }
    }

    /// The value the document publishes as `chatLook`: `None` at Default, and the effective layout
    /// in `messageLayout`, so a renderer never resolves Your messages' implied Left itself.
    pub fn published(&self) -> Option<Self> {
        (!self.is_default()).then(|| Self {
            message_layout: self.effective_layout(),
            ..*self
        })
    }
}

/// Adopts what `settings` names, throwing the cached projections away when the look moved, the
/// same way [`crate::transcript::tool_card::set_enabled`] does: every row's model depends on it.
pub fn apply(view: &mut crate::state::TranscriptViewState, settings: &ChatLookSettings) {
    let next = view.chat_look.with_settings(settings);
    if next != view.chat_look {
        view.chat_look = next;
        view.invalidate();
    }
}

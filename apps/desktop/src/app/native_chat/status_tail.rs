//! Status line in the chat (`chatStatusInTranscript`, gx-chat-core `transcript/chat_look.rs`): the
//! working card's line (working_card.rs) drawn as the transcript's last row, in the quiet lines'
//! type, instead of the capsule above the chat box. The row is not in the core's document: the list
//! holds one row more than the transcript has while the line shows, so following the tail keeps it
//! in view like the newest message. At Default nothing here runs.

use super::{appearance::ChatAppearance, state::NativeChatView, working_card::WorkingLineLook};
use gpui::prelude::FluentBuilder as _;
use gpui::{
    AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

impl NativeChatView {
    /// Whether the transcript ends with the status row: Status line in the chat is on and the
    /// working card would show (the core never sends its worked phase then, so the row goes when
    /// the turn ends and that reply's own Worked line keeps the time).
    fn status_tail_wanted(&self, p: &ChatAppearance) -> bool {
        p.chat_look.status_in_transcript
            && !self.items.is_empty()
            && self.snapshot["workingCard"]["visible"] == true
    }

    /// Adds or removes the status row after the transcript's last row, before the rows are drawn,
    /// and draws them again this frame when it moved.
    pub(super) fn sync_status_tail(&mut self, p: &ChatAppearance, window: &mut Window) {
        let wanted = self.status_tail_wanted(p);
        if wanted == self.status_tail {
            return;
        }
        let rows = self.items.len();
        self.list.splice(
            rows..rows + usize::from(self.status_tail),
            usize::from(wanted),
        );
        self.status_tail = wanted;
        if let Some(host) = &self.transcript_host {
            window.render_view_this_frame(host.entity_id());
        }
    }

    /// CDXC:SessionChat 2026-11-07 DECISION:
    /// Status line in the chat. User: "the line that has the status that shows at the top of the input bar, that I really enjoyed in Claude to be part of the messages". On, the capsule above the chat box is gone and, while the agent works (or a compaction runs), its line is the transcript's last row: the animated spark, the label the capsule would show (a tool's folders dimmed) and the activity's progress and time, in the quiet lines' type (12.5px on a 20px line, dim). When the turn ends the row goes and that reply's Worked line keeps its time. The pills and armed timers stay in the chat box's strip. This is an opt-in alternative to the 2026-10-18 DECISION in packages/gx-chat-core/src/extras/working_card.rs, which still describes the default: "all of these things need to be advanced mode and if we don't enable anything then it should remain to look like it does now".
    ///
    /// CDXC:SessionChat 2026-11-07 WHY:
    /// The spark animates inside the cached transcript view, so while this row is on screen each indicator frame lays out the visible rows again, the cost the default card avoids by living in the composer's stack (the 2026-10-18 WHY on `render_working_card`). It is the price of the line scrolling with the conversation, which only this option asks for; scrolled out of view the spark asks for no frames.
    pub(super) fn status_tail_row(&mut self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        let p = ChatAppearance::current(&self.snapshot).on_window_glass(
            crate::app::helpers::window_glass_active_for(self.main_window),
        );
        let s = p.scale;
        let rows = self.list.item_count();
        self.detail_rows.start_row(true, index, rows);
        self.detail_rows.end_row();
        // A row the list still holds for a moment after the line went (the next render takes it
        // out) draws nothing.
        if !self.status_tail_wanted(&p) {
            return div().into_any_element();
        }
        let card = self.snapshot["workingCard"].clone();
        let header = self.working_card_header(&card, WorkingLineLook::Quiet, &p, cx);
        let line = div()
            .id("chat-status-tail")
            .role(gpui::Role::Status)
            .aria_label(super::transcript::text(&card, "ariaLabel"))
            .w_full()
            .min_w_0()
            // The spark's glow stays within the line, as the capsule clips it.
            .overflow_hidden()
            .text_color(p.muted)
            // Under Names the line sits in under the agent's name like the agent's rows.
            .when(p.names(), |line| {
                line.pl(px(super::user_message_look::NAMES_INDENT * s))
            })
            .child(header);
        div()
            .w_full()
            .flex()
            .justify_center()
            .child(self.transcript_row_column(&p).child(line))
            .into_any_element()
    }
}

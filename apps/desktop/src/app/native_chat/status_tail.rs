//! Status line in the chat (`chatStatusInTranscript`, gx-chat-core `transcript/chat_look.rs`): the
//! working card's line (working_card.rs) drawn as the transcript's last row, in the quiet lines'
//! type, instead of the capsule above the chat box. The row is not in the core's document: the list
//! holds one row more than the transcript has while the line shows, so following the tail keeps it
//! in view like the newest message. At Default nothing here runs.

use super::{
    appearance::ChatAppearance,
    state::NativeChatView,
    working_card::{PaintedSlot, WorkingLineLook, working_line_lead},
};
use gpui::prelude::FluentBuilder as _;
use gpui::{
    AnyElement, App, AvailableSpace, Bounds, ContentMask, Context, Element, GlobalElementId,
    InspectorElementId, InteractiveElement as _, IntoElement, LayoutId, ParentElement as _, Pixels,
    StatefulInteractiveElement as _, Styled as _, Window, canvas, div, px,
};
use serde_json::Value;
use std::{cell::Cell, rc::Rc};

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
        if !wanted {
            self.working_card.tail_lead.set(None);
            self.working_card.tail_line.set(None);
        }
        if let Some(host) = &self.transcript_host {
            window.render_view_this_frame(host.entity_id());
        }
    }

    /// CDXC:SessionChat 2026-11-07 DECISION:
    /// Status line in the chat. User: "the line that has the status that shows at the top of the input bar, that I really enjoyed in Claude to be part of the messages". On, the capsule above the chat box is gone and, while the agent works (or a compaction runs), its line is the transcript's last row: the animated spark, the label the capsule would show (a tool's folders dimmed) and the activity's progress and time, in the quiet lines' type (12.5px on a 20px line, dim). When the turn ends the row goes and that reply's Worked line keeps its time. The pills and armed timers stay in the chat box's strip. This is an opt-in alternative to the 2026-10-18 DECISION in packages/gx-chat-core/src/extras/working_card.rs, which still describes the default: "all of these things need to be advanced mode and if we don't enable anything then it should remain to look like it does now".
    ///
    /// CDXC:SessionChat 2026-11-07 WHY:
    /// The row lives in the cached transcript view, so nothing in it may animate: an indicator there notifies that view, which marks the chat dirty and lays out every visible row again on each frame, the cost the default card avoids by living in the composer's stack (the 2026-10-18 WHY on `render_working_card`, the 2026-09-23 one in transcript_host.rs). The row keeps the lead's place empty and records where it was painted; the chat draws the animated spark (or an activity's loader) over that place after the transcript (`render_status_tail_lead`), so its frames are the chat's animation-only frames and the rows stay cached. A compaction's indeterminate progress track stays still in the row for the same reason.
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
            .aria_label(status_tail_aria_label(&card))
            .w_full()
            .min_w_0()
            .relative()
            // The spark's glow stays within the line, as the capsule clips it.
            .overflow_hidden()
            .text_color(p.muted)
            // Under Names the line sits in under the agent's name like the agent's rows.
            .when(p.names(), |line| {
                line.pl(px(super::user_message_look::NAMES_INDENT * s))
            })
            .child(header)
            .child({
                let painted = self.working_card.tail_line.clone();
                canvas(
                    move |bounds, _, _| painted.set(Some(bounds)),
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0()
            });
        div()
            .w_full()
            .flex()
            .justify_center()
            .child(self.transcript_row_column(&p).child(line))
            .into_any_element()
    }

    /// The status row's animated lead, drawn over the empty place the row painted for it and
    /// clipped to that row inside the transcript's visible area. It is visual only: no handlers,
    /// no hit test, so every press goes to the transcript under it. Placed after the transcript so
    /// a row painted this frame is where it draws.
    pub(super) fn render_status_tail_lead(&self, p: &ChatAppearance) -> Option<AnyElement> {
        if !self.status_tail {
            return None;
        }
        let reduced_motion = crate::app::helpers::gpui_macos_reduce_motion_enabled();
        let lead = working_line_lead(&self.snapshot["workingCard"], p, reduced_motion)?;
        Some(
            PaintedAtSlot {
                slot: self.working_card.tail_lead.clone(),
                line: self.working_card.tail_line.clone(),
                child: lead,
                clip: None,
            }
            .into_any_element(),
        )
    }
}

/// The row's label and the activity's numbers, for assistive clients.
fn status_tail_aria_label(card: &Value) -> String {
    let activity = &card["activity"];
    let percent = activity["percent"]
        .as_f64()
        .map(|percent| format!("{percent}%"));
    [card["label"].as_str().map(str::to_owned)]
        .into_iter()
        .chain(["elapsedLabel", "tokens"].map(|key| activity[key].as_str().map(str::to_owned)))
        .chain([percent])
        .flatten()
        .collect::<Vec<_>>()
        .join(", ")
}

/// Takes no room where it sits, and lays out and paints `child` at the place `slot` last recorded,
/// inside that clip and the `line` it belongs to.
struct PaintedAtSlot {
    slot: PaintedSlot,
    line: Rc<Cell<Option<Bounds<Pixels>>>>,
    child: AnyElement,
    /// The clip the child was prepainted in, for its paint.
    clip: Option<Bounds<Pixels>>,
}

impl Element for PaintedAtSlot {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let style = gpui::Style {
            position: gpui::Position::Absolute,
            ..Default::default()
        };
        (window.request_layout(style, None, cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let (Some((slot, mask)), Some(line)) = (self.slot.get(), self.line.get()) else {
            return;
        };
        let clip = mask
            .intersect(&line)
            .intersect(&window.content_mask().bounds);
        self.clip = Some(clip);
        window.with_content_mask(Some(ContentMask { bounds: clip }), |window| {
            self.child.prepaint_as_root(
                slot.origin,
                slot.size.map(AvailableSpace::Definite),
                window,
                cx,
            );
        });
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let Some(clip) = self.clip else {
            return;
        };
        window.with_content_mask(Some(ContentMask { bounds: clip }), |window| {
            self.child.paint(window, cx);
        });
    }
}

impl IntoElement for PaintedAtSlot {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

//! Agent work Grouped (`chatAgentWorkStyle` `grouped`, `transcript/chat_look.rs`): everything the
//! agent did for one message of the user's reads as one line, "Worked for 1m 16s · 9 tool calls ·
//! 2 files", and the agent's messages for that reply follow it.
//!
//! The finished turns the projection folds (`completed-work`) are merged after they are built
//! (`presentation::build_items`): each run of them with no row between, which is one prompt's turn
//! and the turns the agent's wake-ups after it opened, becomes one `grouped-work` row, and a turn
//! alone becomes one too, so the renderers draw a single style. The running turn's rows and every
//! other row stay where they are, after the line.
//!
//! CDXC:SessionChat 2026-11-07 WHY: the line's time adds up each merged turn's own time (`presentation::turn_worked_ms`), not their labels: a prompt's label already runs on through the OMP wake-ups after it (`turn_worked_label`), so adding the labels counted those turns twice.
//!
//! CDXC:SessionChat 2026-11-07 SEE-ALSO: apps/desktop/src/app/native_chat/ (desktop and web) and apps/mobile/app/src/chat/ (phone) draw `grouped-work`; `extras/transcript_search.rs` reads its fold for Find; `transcript/rows.rs` (`without_worked_time`) relabels the newest one "Previous turn".

use serde_json::Value;

use crate::document::{GroupedTurn, TranscriptItem};
use crate::transcript::file_change_rows::file_change_card;
use crate::transcript::read_images::{images_stat, push_once};
use crate::transcript::simple::{edit_label, edited_and_failed_paths};
use crate::transcript::turns::worked_duration_label;
use crate::transcript::work_summary::counted_work_stats;

/// `items` with each run of `completed-work` rows folded into one `grouped-work` row.
pub fn group_work(items: Vec<TranscriptItem>) -> Vec<TranscriptItem> {
    let mut grouped = Vec::with_capacity(items.len());
    let mut run: Vec<TranscriptItem> = Vec::new();
    for item in items {
        if matches!(item, TranscriptItem::CompletedWork { .. }) {
            run.push(item);
            continue;
        }
        if !run.is_empty() {
            grouped.push(merge(std::mem::take(&mut run)));
        }
        grouped.push(item);
    }
    if !run.is_empty() {
        grouped.push(merge(run));
    }
    grouped
}

/// One `grouped-work` row for a run of `completed-work` rows, oldest first.
fn merge(run: Vec<TranscriptItem>) -> TranscriptItem {
    let mut id = String::new();
    let mut worked_ms = 0;
    let mut tool_calls = 0;
    let mut changed: Vec<String> = Vec::new();
    let mut files: Vec<Value> = Vec::new();
    let mut expandable = false;
    let mut work = Vec::new();
    let mut read_images: Vec<Value> = Vec::new();
    let mut turns = Vec::with_capacity(run.len());
    for item in run {
        let TranscriptItem::CompletedWork {
            id: turn_id,
            files: turn_files,
            expandable: turn_expandable,
            deferred,
            work: turn_work,
            questions,
            sent_messages,
            workflows,
            artifacts,
            final_message,
            read_images: turn_images,
            tally,
            ..
        } = item
        else {
            continue;
        };
        if id.is_empty() {
            id = format!("grouped:{turn_id}");
        }
        worked_ms += tally.worked_ms.unwrap_or(0);
        tool_calls += tally.tool_calls;
        for path in tally.changed {
            if !changed.contains(&path) {
                changed.push(path);
            }
        }
        files.extend(turn_files);
        expandable |= turn_expandable;
        work.extend(turn_work);
        for image in turn_images {
            push_once(&mut read_images, image);
        }
        turns.push(GroupedTurn {
            id: turn_id,
            deferred,
            questions,
            sent_messages,
            workflows,
            artifacts,
            final_message,
        });
    }
    let (_, failed) = edited_and_failed_paths(files.iter().filter_map(|file| {
        let path = file.get("path").and_then(Value::as_str)?;
        Some((path, file.get("failed") == Some(&Value::Bool(true))))
    }));
    let failed_files = failed.len();
    let changed_files = changed.len();
    let mut stats = counted_work_stats(tool_calls, changed_files);
    // The strip's "3 images" comes first, as on a finished turn's card (`bubble_read_images`).
    if !read_images.is_empty() {
        stats.insert(0, images_stat(read_images.len()));
    }
    TranscriptItem::GroupedWork {
        id,
        // Plain "Worked" when no merged turn had a length to show.
        label: worked_duration_label(Some(0), Some(worked_ms)),
        stats,
        files_card: file_change_card(&files),
        files_label: format!(
            "{changed_files} {} changed",
            if changed_files == 1 { "file" } else { "files" }
        ),
        simple_files_label: edit_label(changed_files, failed_files),
        files,
        expandable,
        work,
        turns,
        read_images,
    }
}

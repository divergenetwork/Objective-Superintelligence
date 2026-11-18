//! A voter's first tool: the next decision in its queue.

use diverge_sdk::daemon::key;
use rmcp::model::ContentBlock;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::failure::Failure;
use crate::server::State;
use crate::store::queue;
use crate::store::queue::Read;

/// The tool's name.
pub const NAME: &str = "get";

/// The tool's description, as `tools/list` states it.
pub const DESCRIPTION: &str = "Take the next structured decision out of your queue. It is returned as JSON, with `id`, `content` \
and `questions`; an image, audio or a file in it is returned as a part of its own, in its place within the JSON. Each \
decision is returned once: the next call returns the one after it, or `null` when your queue is empty. Answer a decision \
with `vote`, giving its `id`.";

/// The tool's parameters: none.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {}

/// The caller's next decision, as the parts of one message; or the one
/// text part `null` where its queue is empty. The entry is marked read
/// as it is handed over, so no call returns it again.
pub async fn call(state: &State, caller: &key::Agent) -> Result<Vec<ContentBlock>, Failure> {
    let pool = state.store.pool(state.daemon.client()).await?;
    match queue::next(pool, &caller.template, caller.index).await? {
        Some(read) => message(read),
        None => Ok(vec![ContentBlock::text("null")]),
    }
}

/// A decision as the voter reads it: one message, in the modalities
/// the decision came in. The rules:
///
/// 1. **The message is the decision as pretty JSON**: an object with
///    `id`, the entry's id as eight hex digits; `content`; and
///    `questions`; the last two exactly as the call that put the
///    decision carried them.
/// 2. **A text block stays in the JSON**, as it is.
/// 3. **Every other block is a part of its own.** An image, audio, a
///    file or a link is taken out of the JSON and sent as a part of
///    its kind, where it stood: the JSON before it is one text part,
///    and the JSON after it goes on in the next. So the parts, read in
///    order, are the JSON with each such block shown in its place.
/// 4. **A block is taken out wherever a decision holds blocks**: in
///    `content`, in a question's `instructions`, in a choice option's
///    `description`, and in a score's `levels`.
///
/// A block is taken out by putting a marker string in its place before
/// the JSON is written, and cutting the written text at each marker,
/// quotes included. The marker is one the decision's own text does not
/// hold: see [`marker`].
fn message(read: Read) -> Result<Vec<ContentBlock>, Failure> {
    let unreadable = |error: serde_json::Error| Failure::Store(error.to_string());
    let mut content: Value = serde_json::from_str(&read.content).map_err(unreadable)?;
    let mut questions: Value = serde_json::from_str(&read.questions).map_err(unreadable)?;
    let marker = marker(&read);
    let mut taken = Vec::new();
    // Rule 4.
    lift(&mut content, &marker, &mut taken);
    for question in questions.as_array_mut().into_iter().flatten() {
        if let Some(instructions) = question.get_mut("instructions") {
            lift(instructions, &marker, &mut taken);
        }
        for option in question.get_mut("options").and_then(Value::as_array_mut).into_iter().flatten() {
            if let Some(description) = option.get_mut("description") {
                lift(description, &marker, &mut taken);
            }
        }
        for level in question.get_mut("levels").and_then(Value::as_array_mut).into_iter().flatten() {
            lift(level, &marker, &mut taken);
        }
    }
    // Rule 1.
    let text = serde_json::to_string_pretty(&json!({ "id": format!("{:08x}", read.id), "content": content, "questions": questions })).map_err(unreadable)?;
    // Rule 3: where each marker stands in the text, in the text's order.
    let mut cuts = Vec::new();
    for position in 0..taken.len() {
        let quoted = format!("\"{marker}{position}\"");
        if let Some(start) = text.find(&quoted) {
            cuts.push((start, quoted.len(), position));
        }
    }
    cuts.sort_unstable();
    let mut taken: Vec<Option<ContentBlock>> = taken.into_iter().map(Some).collect();
    let mut parts = Vec::new();
    let mut from = 0;
    for (start, length, position) in cuts {
        parts.push(ContentBlock::text(&text[from..start]));
        parts.extend(taken[position].take());
        from = start + length;
    }
    parts.push(ContentBlock::text(&text[from..]));
    Ok(parts)
}

/// Take every block of `blocks` that is not text out, leaving in its
/// place the string `marker` followed by its number among the blocks
/// taken. `blocks` is a JSON array of content blocks; anything else,
/// and any element that does not read as a content block, is left as
/// it is.
fn lift(blocks: &mut Value, marker: &str, taken: &mut Vec<ContentBlock>) {
    for block in blocks.as_array_mut().into_iter().flatten() {
        if block.get("type").and_then(Value::as_str) == Some("text") {
            continue;
        }
        if let Ok(read) = serde_json::from_value::<ContentBlock>(block.clone()) {
            *block = Value::String(format!("{marker}{}", taken.len()));
            taken.push(read);
        }
    }
}

/// A marker the decision's text does not hold: `@@part-N-`, with the
/// least `N` for which neither the content nor the questions, as the
/// JSON text they are stored as, holds it. The stored text is as
/// `serde_json` writes it, which is how the message is written, so a
/// marker absent from the one is absent from the other.
fn marker(read: &Read) -> String {
    let mut nonce = 0u64;
    loop {
        let marker = format!("@@part-{nonce}-");
        if !read.content.contains(&marker) && !read.questions.contains(&marker) {
            return marker;
        }
        nonce += 1;
    }
}

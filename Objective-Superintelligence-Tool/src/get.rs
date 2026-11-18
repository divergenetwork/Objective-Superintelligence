//! A voter's first tool: the next decision in its queue.

use diverge_sdk::daemon::key;
use diverge_sdk::shared::containers::structured_decisions::Question;
use rmcp::model::ContentBlock;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::failure::Failure;
use crate::server::State;
use crate::store::queue;

/// The tool's name.
pub const NAME: &str = "get";

/// The tool's description, as `tools/list` states it.
pub const DESCRIPTION: &str = "Take the next structured decision out of your queue. It is returned as one JSON object, with \
`id`, `content` and `questions`, written across the parts of the result: read them in order, as one text. `content`, each \
question's `instructions`, each option's `description` and each of a score's `levels` is a quoted string, and an image, audio \
or a file that belongs to it is a part of its own, standing inside the quotes where it belongs. Each decision is returned \
once: the next call returns the one after it, or `null` when your queue is empty. Answer a decision with `vote`, giving its `id`.";

/// The tool's parameters: none.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {}

/// The caller's next decision, as the parts of one message; or the one
/// text part `null` where its queue is empty. The entry is marked read
/// as it is handed over, so no call returns it again.
pub async fn call(state: &State, caller: &key::Agent) -> Result<Vec<ContentBlock>, Failure> {
    let pool = state.store.pool(state.daemon.client()).await?;
    let Some(read) = queue::next(pool, &caller.template, caller.index).await? else {
        return Ok(vec![ContentBlock::text("null")]);
    };
    let content: Vec<ContentBlock> = serde_json::from_str(&read.content).map_err(unreadable)?;
    let questions: Vec<Question> = serde_json::from_str(&read.questions).map_err(unreadable)?;
    message(read.id, &content, &questions)
}

/// A stored decision that does not read, or a string that could not be
/// written, as the call's failure.
fn unreadable(error: serde_json::Error) -> Failure {
    Failure::Store(error.to_string())
}

/// A message being written: the parts made so far, and the JSON filler
/// written since the last of them, which is the next text part.
///
/// The message is one pretty JSON object, written across parts. The
/// rules:
///
/// 1. **Filler is text parts.** Keys, quotes, commas, brackets and
///    indentation are written with [`filler`](Message::filler) and
///    [`string`](Message::string), and gather into one text part until
///    a block is sent.
/// 2. **A list of content blocks is one quoted string.** Its opening
///    quote ends the filler before it, and its closing quote begins
///    the filler after it: see [`blocks`](Message::blocks).
/// 3. **The blocks stand inside the quotes, each a part of its own**,
///    in order, exactly as the decision carried them, but for one
///    thing: a text block's text is escaped as a JSON string's is, so
///    that the parts read in order are valid JSON.
#[derive(Default)]
struct Message {
    parts: Vec<ContentBlock>,
    filler: String,
}

impl Message {
    /// Write JSON filler.
    fn filler(&mut self, text: &str) {
        self.filler.push_str(text);
    }

    /// Write `text` as a JSON string, quoted and escaped, as filler:
    /// a scalar, and no list of blocks.
    fn string(&mut self, text: &str) -> Result<(), Failure> {
        self.filler.push_str(&serde_json::to_string(text).map_err(unreadable)?);
        Ok(())
    }

    /// Write one list of content blocks as one quoted string: rules 2
    /// and 3.
    fn blocks(&mut self, blocks: &[ContentBlock]) -> Result<(), Failure> {
        self.filler.push('"');
        self.parts.push(ContentBlock::text(std::mem::take(&mut self.filler)));
        for block in blocks {
            let mut block = block.clone();
            if let ContentBlock::Text(text) = &mut block {
                text.text = escaped(&text.text)?;
            }
            self.parts.push(block);
        }
        self.filler.push('"');
        Ok(())
    }

    /// The message's parts: those made, and the filler after the last
    /// of them.
    fn end(mut self) -> Vec<ContentBlock> {
        self.parts.push(ContentBlock::text(self.filler));
        self.parts
    }
}

/// `text` as it stands inside a JSON string: escaped, and without the
/// quotes around it. The escaping is `serde_json`'s own, which writes
/// a string quoted; the two quotes are its first and last characters.
fn escaped(text: &str) -> Result<String, Failure> {
    let quoted = serde_json::to_string(text).map_err(unreadable)?;
    Ok(quoted[1..quoted.len() - 1].to_owned())
}

/// What begins an element of a list written one element per line at
/// `indent`: the line break and the indent, after a comma for every
/// element but the first.
fn element(first: bool, indent: &str) -> String {
    format!("{}\n{indent}", if first { "" } else { "," })
}

/// What ends a list whose own line is at `indent`: the bracket, on a
/// line of its own unless the list is empty.
fn closed(empty: bool, indent: &str) -> String {
    if empty { "]".to_owned() } else { format!("\n{indent}]") }
}

/// A decision as the voter reads it: one pretty JSON object, two
/// spaces to a level and one member to a line, written as [`Message`]
/// writes one. The object:
///
/// 1. `id`: the entry's id, as eight hex digits.
/// 2. `content`: the decision's content.
/// 3. `questions`: each question, with its `type`, its `id` and its
///    `instructions`; a choice's `options`, each with its `value` and,
///    where it has one, its `description`; and a score's `levels`.
///
/// `content`, `instructions`, a `description` and each level are lists
/// of content blocks, and each is written as one quoted string holding
/// its blocks.
fn message(id: u32, content: &[ContentBlock], questions: &[Question]) -> Result<Vec<ContentBlock>, Failure> {
    let mut message = Message::default();
    message.filler("{\n  \"id\": ");
    message.string(&format!("{id:08x}"))?;
    message.filler(",\n  \"content\": ");
    message.blocks(content)?;
    message.filler(",\n  \"questions\": [");
    for (position, question) in questions.iter().enumerate() {
        message.filler(&element(position == 0, "    "));
        message.filler("{\n      \"type\": ");
        message.string(match question {
            Question::Noul { .. } => "noul",
            Question::Choice { .. } => "choice",
            Question::Score { .. } => "score",
        })?;
        message.filler(",\n      \"id\": ");
        message.string(question.id())?;
        message.filler(",\n      \"instructions\": ");
        message.blocks(question.instructions())?;
        match question {
            Question::Noul { .. } => {}
            Question::Choice { options, .. } => {
                message.filler(",\n      \"options\": [");
                for (position, option) in options.iter().enumerate() {
                    message.filler(&element(position == 0, "        "));
                    message.filler("{\n          \"value\": ");
                    message.string(&option.value)?;
                    if let Some(description) = &option.description {
                        message.filler(",\n          \"description\": ");
                        message.blocks(description)?;
                    }
                    message.filler("\n        }");
                }
                message.filler(&closed(options.is_empty(), "      "));
            }
            Question::Score { levels, .. } => {
                message.filler(",\n      \"levels\": [");
                for (position, level) in levels.iter().enumerate() {
                    message.filler(&element(position == 0, "        "));
                    message.blocks(level)?;
                }
                message.filler(&closed(levels.is_empty(), "      "));
            }
        }
        message.filler("\n    }");
    }
    message.filler(&closed(questions.is_empty(), "  "));
    message.filler("\n}");
    Ok(message.end())
}

//! What one agent made of a decision, as the tool returns it.

use diverge_sdk::shared::containers::structured_decisions::Answer;
use indexmap::IndexMap;
use serde::Deserialize;

/// The tool's result: one outcome per agent, in the order of the
/// agents the tool was made with, which is the order of this agent's
/// own arguments.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Results {
    /// The outcomes.
    pub results: Vec<Outcome>,
}

/// What one agent made of a decision: its answers, or that it gave
/// none. On the wire `{"answer":{…}}` or the string `"failure"`,
/// which says nothing more.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// The agent's answers, each by its question's id.
    Answer(IndexMap<String, Answer>),
    /// The agent gave no answer.
    Failure,
}

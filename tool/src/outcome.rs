//! What one agent made of a decision.

use diverge_sdk::shared::containers::structured_decisions::Answer;
use indexmap::IndexMap;
use serde::Serialize;

/// What one agent made of a decision: its answers, or that it gave
/// none. On the wire `{"answer":{…}}` or the string `"failure"`.
///
/// A failure says nothing more, by design: the agent refused the
/// decision, could not be reached, never took it up, or went idle
/// without answering, and to the caller these are one thing — an
/// agent with no answer to weigh.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// The agent's answers, each by its question's id, as the agent
    /// gave them: nothing is checked against the questions here.
    Answer(IndexMap<String, Answer>),
    /// The agent gave no answer.
    Failure,
}

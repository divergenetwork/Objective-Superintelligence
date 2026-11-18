//! Why a tool call did not do what it was asked.

use std::fmt;

/// A tool call's failure, said to the caller as the tool's error
/// result. These are failures of the CALL: one agent's failure to
/// answer is not one, and is that agent's
/// [`Outcome::Failure`](crate::outcome::Outcome::Failure) in a call
/// that succeeded.
#[derive(Debug)]
pub enum Failure {
    /// The tool's parameters are not what it takes.
    Parameters(String),
    /// The daemon could not be reached.
    Daemon(String),
    /// The database could not be reached, or refused a statement.
    Store(String),
    /// The caller is not an agent the daemon attested, so it has no
    /// queue to read and nothing to vote on.
    NoQueue,
    /// The caller has no decision by this id: it never had one, or the
    /// decision closed, or the vote on it is already in.
    NoDecision(String),
    /// The vote does not answer the decision's questions, each once
    /// and each by its kind.
    Vote(String),
    /// The call's own task ended before it answered.
    Ended(String),
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Failure::Parameters(why) => write!(f, "the parameters are not what this tool takes: {why}"),
            Failure::Daemon(why) => write!(f, "the daemon could not be reached: {why}"),
            Failure::Store(why) => write!(f, "the queue could not be read or written: {why}"),
            Failure::NoQueue => write!(f, "only an agent has a queue, and the tool could not tell which agent is calling"),
            Failure::NoDecision(id) => write!(f, "you have no open decision {id}: it closed, or your vote on it is already in"),
            Failure::Vote(why) => write!(f, "the vote was not taken, and may be made again: {why}"),
            Failure::Ended(why) => write!(f, "the call ended before it answered: {why}"),
        }
    }
}

impl std::error::Error for Failure {}

impl From<sqlx::Error> for Failure {
    fn from(error: sqlx::Error) -> Self {
        Failure::Store(error.to_string())
    }
}

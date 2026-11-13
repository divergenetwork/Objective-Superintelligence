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
    /// The tool was called before it was registered, so it has no
    /// agents to ask.
    Unregistered,
    /// The daemon could not be reached.
    Daemon(String),
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Failure::Parameters(why) => write!(f, "the parameters are not what this tool takes: {why}"),
            Failure::Unregistered => write!(f, "the tool is not registered yet, and has no agents to ask"),
            Failure::Daemon(why) => write!(f, "the daemon could not be reached: {why}"),
        }
    }
}

impl std::error::Error for Failure {}

//! Why a decision got no answer.

use std::fmt;

/// Why a decision the agent took up ends with no answer: said in the
/// run's one fatal notification, in place of the response.
#[derive(Debug)]
pub enum Failure {
    /// The proxy's MCP could not be reached, or the tool list could
    /// not be read.
    Mcp(String),
    /// The dependency tool is not among the tools the agent has.
    NoTool,
    /// The tool answered the call with its own error.
    Tool(String),
    /// The tool's result is not the result it gives: not an outcome
    /// per agent, in the agents' order.
    Result(String),
    /// Not one agent gave an answer that fits the questions, so there
    /// is nothing to average.
    NoAnswers,
    /// The answers could not be averaged: a number among them is
    /// beyond what a decimal can hold.
    Overflow,
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Failure::Mcp(why) => write!(f, "the agent's tools could not be reached: {why}"),
            Failure::NoTool => write!(f, "the agent's dependency tool is not among its tools"),
            Failure::Tool(why) => write!(f, "the dependency tool failed: {why}"),
            Failure::Result(why) => write!(f, "the dependency tool's result is not what it gives: {why}"),
            Failure::NoAnswers => write!(f, "none of the agents asked gave an answer that fits the questions"),
            Failure::Overflow => write!(f, "the agents' answers hold a number too large to average"),
        }
    }
}

impl std::error::Error for Failure {}

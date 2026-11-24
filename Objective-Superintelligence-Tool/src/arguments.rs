//! What the tool takes as its arguments: the image of the agent it
//! serves `structured_decisions` to.

use serde::{Deserialize, Serialize};

/// The arguments the tool is made with.
///
/// # How it runs
///
/// As a dependency tool. What it serves depends on its caller, as the
/// daemon attests it on every request: an agent whose image digest is
/// [`superintelligence_agent_digest`](Self::superintelligence_agent_digest)
/// is served `structured_decisions`, which puts a decision to the
/// agents the call lists; every other caller is served `get` and
/// `vote`, the next decision in its queue and its vote on one. See
/// [`caller`](crate::caller).
///
/// # Who gives the argument
///
/// An Objective-Superintelligence agent declares the tool itself, and
/// gives its own image's digest, which it is told at its
/// registration. A voter declares the tool with the same argument:
/// the tool is one for every agent that declares it with the same
/// arguments, so only then is the voter's tool the one the agent
/// calls, and the queue it reads the one the agent's decisions are
/// put in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Arguments {
    /// The image of the Objective-Superintelligence agent this tool
    /// serves `structured_decisions` to: its manifest digest,
    /// `<algorithm>:<hex>`. An agent of any other image is served
    /// `get` and `vote`.
    pub superintelligence_agent_digest: String,
}

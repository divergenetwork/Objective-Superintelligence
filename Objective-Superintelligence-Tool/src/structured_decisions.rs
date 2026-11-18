//! The one tool: a decision put to every agent.

use diverge_sdk::shared::containers::structured_decisions::Question;
use futures_util::future::join_all;
use rmcp::model::ContentBlock;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::agent::Agent;
use crate::ask::ask;
use crate::daemon::Daemon;
use crate::failure::Failure;
use crate::outcome::Outcome;

/// The tool's name.
pub const NAME: &str = "structured_decisions";

/// The tool's description, as `tools/list` states it.
pub const DESCRIPTION: &str = "Put one structured decision (a prompt and typed questions) to every agent listed in `agents`, and \
return what each answered. `results` has one entry per agent, in the order `agents` lists them: an object with one member \
`answer`, the agent's answers by question id, or the string `failure` for an agent that gave none.";

/// The tool's parameters: the agents asked, and a decision, as
/// `agents message` takes one.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    /// The agents asked, in the order their answers are returned in.
    /// At least one.
    #[schemars(length(min = 1))]
    pub agents: Vec<Agent>,
    /// What is being decided on: the prompt every question is about.
    pub content: Vec<ContentBlock>,
    /// The questions, each with its own id.
    pub questions: Vec<Question>,
}

/// The tool's answer.
#[derive(Debug, Clone, Serialize)]
pub struct Results {
    /// One outcome per agent, in the order of the call's `agents`.
    pub results: Vec<Outcome>,
}

/// Put the decision to every agent at once, and gather what each made
/// of it. The agents do not wait on one another, and one's failure is
/// its own outcome and not the call's.
pub async fn call(daemon: &Daemon, parameters: Parameters) -> Result<Results, Failure> {
    if parameters.agents.is_empty() {
        return Err(Failure::Parameters("`agents` lists no agent, and there must be at least one".to_owned()));
    }
    let handle = daemon.handle().await?;
    let results = join_all(parameters.agents.iter().map(|agent| ask(daemon, &handle, agent, &parameters.content, &parameters.questions))).await;
    Ok(Results { results })
}

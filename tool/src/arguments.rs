//! What the tool takes as its arguments: the agents it asks.

use diverge_sdk::daemon::reference;
use diverge_sdk::shared::permission::agents::Key;
use serde::{Deserialize, Serialize};

/// The arguments the tool is made with: the agents every decision is
/// put to.
///
/// # How it runs
///
/// As a dependency tool of the Objective-Superintelligence agent, which
/// declares it with its own agents, weights left out. The tool asks
/// the daemon for one thing: `message` over exactly the agents
/// listed here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Arguments {
    /// The agents asked, in the order their answers are returned in.
    /// At least one.
    #[schemars(length(min = 1))]
    pub agents: Vec<Agent>,
}

/// One agent asked: the template it was made from and its index among
/// the agents made from that template, which together name it once
/// and for all.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Agent {
    /// The agent's template, by id.
    pub template: String,
    /// The agent's index among the agents made from that template.
    pub index: u64,
}

impl Agent {
    /// The agent, as a request to the daemon names it.
    pub fn reference(&self) -> reference::Agent {
        reference::Agent::TemplateIndex {
            template: self.template.clone(),
            index: self.index,
        }
    }

    /// The agent, as a grant lists it.
    pub fn key(&self) -> Key {
        Key {
            template: self.template.clone(),
            index: self.index,
        }
    }
}

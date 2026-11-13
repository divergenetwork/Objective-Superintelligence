//! What the agent takes as its arguments: the agents it asks, and how
//! much each one's answer counts.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// The arguments the agent is made with: the agents every decision is
/// put to, each with the weight its answer is averaged by.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Arguments {
    /// The agents asked. At least one.
    #[schemars(length(min = 1))]
    pub agents: Vec<Agent>,
}

/// One agent asked: the template it was made from and its index among
/// the agents made from that template, which together name it once
/// and for all, and its weight.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Agent {
    /// The agent's template, by id.
    pub template: String,
    /// The agent's index among the agents made from that template.
    pub index: u64,
    /// How much this agent's answer counts beside the others': a
    /// number greater than zero. Only the proportions matter: weights
    /// of 1 and 3 average as weights of 0.25 and 0.75 do.
    #[schemars(extend("exclusiveMinimum" = 0))]
    pub weight: f64,
}

impl Arguments {
    /// Each agent's weight as the decimal the averaging is done in,
    /// in the arguments' order; or why the arguments cannot be taken.
    ///
    /// The rules, beyond the shape serde holds them to:
    ///
    /// 1. There is at least one agent.
    /// 2. Every weight is greater than zero, and a number a decimal
    ///    can hold: not infinite, not NaN, not beyond its range.
    pub fn weights(&self) -> Result<Vec<Decimal>, String> {
        if self.agents.is_empty() {
            return Err("`agents` lists no agent, and there must be at least one".to_owned());
        }
        self.agents
            .iter()
            .enumerate()
            .map(|(position, agent)| match Decimal::try_from(agent.weight) {
                Ok(weight) if weight > Decimal::ZERO => Ok(weight),
                _ => Err(format!("the weight of agent {position} is {}, and a weight is a number greater than zero", agent.weight)),
            })
            .collect()
    }
}

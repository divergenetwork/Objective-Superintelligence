//! What the agent takes as its arguments: the image of its tool, the
//! agents it asks, and how much each one's answer counts.

use diverge_sdk::daemon::endpoints::daemons::DaemonId;
use diverge_sdk::shared::containers::request::Image;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Value, json};

/// The arguments the agent is made with: the image of the tool it
/// asks through, and the agents every decision is put to, each with
/// the weight its answer is averaged by.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Arguments {
    /// The image of the Objective-Superintelligence-Tool, which the
    /// agent declares as its one dependency: its digest, and where
    /// its bytes may be fetched, if anywhere.
    ///
    /// It is an argument, and not a part of this image, because the
    /// tool's image holds this image's digest: the tool serves
    /// `structured_decisions` to this image alone. Were this image to
    /// hold the tool's digest too, each digest would be a hash over
    /// the other, and neither image could be built. So the agent is
    /// built first, the tool is built knowing the agent's digest, and
    /// the tool's digest is given here. A tool built for another
    /// agent image serves this agent `get` and `vote` only, and every
    /// decision fails.
    pub tool: Image,
    /// The agents asked. At least one.
    #[schemars(length(min = 1))]
    pub agents: Vec<Agent>,
}

/// One agent asked, of one of two kinds. Internally tagged by `type`,
/// snake case: `"structured_decision"` or `"voter"`. Each kind takes
/// its own members and no other.
///
/// Either kind is named by the template it was made from and its
/// index among the agents made from that template, which together
/// name it once and for all on the daemon that holds it, and carries
/// the weight its answer is averaged by.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Agent {
    /// An agent that takes structured decisions: the decision is sent
    /// to it as a message, and its answer is read from its log. It may
    /// be on another daemon.
    StructuredDecision {
        /// The agent's template, by id.
        template: String,
        /// The agent's index among the agents made from that template.
        index: u64,
        /// The daemon the agent is on, where it is not this one: the
        /// route to it, as a request's `relay` is. The first ID is
        /// this daemon's record of a remote daemon, and each ID after
        /// it is the record the daemon before it keeps of the next;
        /// the agent is on the daemon the route ends at, and
        /// `template` and `index` are that daemon's. Left out for an
        /// agent on this daemon; never empty.
        #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "route")]
        #[schemars(with = "Vec<DaemonId>", length(min = 1))]
        relay: Option<Vec<DaemonId>>,
        /// How much this agent's answer counts beside the others': a
        /// number greater than zero. Only the proportions matter:
        /// weights of 1 and 3 average as weights of 0.25 and 0.75 do.
        #[schemars(extend("exclusiveMinimum" = 0))]
        weight: f64,
    },
    /// An agent that votes: it declares Objective-Superintelligence-Tool
    /// itself, is told that a decision waits in its queue there, reads
    /// it with `get` and answers it with `vote`. It is on this daemon,
    /// always: the tool it votes through is this daemon's, so a voter
    /// has no `relay`.
    Voter {
        /// The agent's template, by id.
        template: String,
        /// The agent's index among the agents made from that template.
        index: u64,
        /// How much this agent's answer counts beside the others': a
        /// number greater than zero. Only the proportions matter:
        /// weights of 1 and 3 average as weights of 0.25 and 0.75 do.
        #[schemars(extend("exclusiveMinimum" = 0))]
        weight: f64,
    },
}

/// A `relay` as the arguments carry one: the route's daemon IDs, one
/// at least. The rules:
///
/// 1. **One spelling of every meaning.** An agent on this daemon has
///    no `relay`. An empty `relay` would say the same, and so would
///    a `null`; both are refused, since the arguments are hashed into
///    the template's id as they are written, and two spellings of
///    one list of agents would be two templates.
/// 2. **Each ID is a daemon ID**, in its one spelling: see
///    [`DaemonId`].
fn route<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Vec<DaemonId>>, D::Error> {
    let route = Vec::<DaemonId>::deserialize(deserializer)?;
    if route.is_empty() {
        return Err(serde::de::Error::custom("`relay` names no daemon: leave it out for an agent on this daemon"));
    }
    Ok(Some(route))
}

impl Agent {
    /// The agent's weight, whichever kind it is.
    pub fn weight(&self) -> f64 {
        match self {
            Agent::StructuredDecision { weight, .. } | Agent::Voter { weight, .. } => *weight,
        }
    }

    /// The agent as the tool's `structured_decisions` takes it: its
    /// kind, its template and index, and its `relay` where it has one.
    /// Its weight is left out: the tool asks, and weighs nothing.
    fn called(&self) -> Value {
        match self {
            Agent::StructuredDecision { template, index, relay: Some(relay), .. } => {
                json!({ "type": "structured_decision", "template": template, "index": index, "relay": relay })
            }
            Agent::StructuredDecision { template, index, relay: None, .. } => json!({ "type": "structured_decision", "template": template, "index": index }),
            Agent::Voter { template, index, .. } => json!({ "type": "voter", "template": template, "index": index }),
        }
    }
}

impl Arguments {
    /// The agents as the tool's `structured_decisions` takes them, in
    /// the arguments' order: see [`Agent::called`].
    pub fn called(&self) -> Vec<Value> {
        self.agents.iter().map(Agent::called).collect()
    }

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
            .map(|(position, agent)| match Decimal::try_from(agent.weight()) {
                Ok(weight) if weight > Decimal::ZERO => Ok(weight),
                _ => Err(format!("the weight of agent {position} is {}, and a weight is a number greater than zero", agent.weight())),
            })
            .collect()
    }
}

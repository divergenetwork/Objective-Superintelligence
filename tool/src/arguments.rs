//! What the tool takes as its arguments: the agents it asks.

use diverge_sdk::daemon::endpoints::daemons::DaemonId;
use diverge_sdk::daemon::reference;
use diverge_sdk::shared::permission::agents::Key;
use diverge_sdk::shared::relay::Relay;
use serde::{Deserialize, Deserializer, Serialize};

/// The arguments the tool is made with: the agents every decision is
/// put to.
///
/// # How it runs
///
/// As a dependency tool of the Objective-Superintelligence agent, which
/// declares it with its own agents, weights left out. The tool asks
/// the daemon for `message` over exactly the agents listed here that
/// are on it, and for `relay` over exactly the remote daemons the
/// routes of the others begin at: see
/// [`permissions`](crate::permissions::permissions).
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
/// and for all on the daemon that holds it; and that daemon, where it
/// is another.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Agent {
    /// The agent's template, by id.
    pub template: String,
    /// The agent's index among the agents made from that template.
    pub index: u64,
    /// The daemon the agent is on, where it is not this one: the route
    /// to it, as a request's `relay` is. The first ID is this
    /// daemon's record of a remote daemon, and each ID after it is the
    /// record the daemon before it keeps of the next; the agent is on
    /// the daemon the route ends at, and `template` and `index` are
    /// that daemon's. Left out for an agent on this daemon; never
    /// empty.
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "route")]
    #[schemars(with = "Vec<DaemonId>", length(min = 1))]
    pub relay: Option<Vec<DaemonId>>,
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
    /// The agent, as a request to the daemon names it.
    pub fn reference(&self) -> reference::Agent {
        reference::Agent::TemplateIndex {
            template: self.template.clone(),
            index: self.index,
        }
    }

    /// The route the request to the agent takes: its `relay`, or none
    /// for an agent on this daemon.
    pub fn relay(&self) -> Option<Relay> {
        self.relay.clone().map(Relay)
    }

    /// The agent, as a grant lists it: meaningful for an agent on this
    /// daemon only, since the pair is the holding daemon's.
    pub fn key(&self) -> Key {
        Key {
            template: self.template.clone(),
            index: self.index,
        }
    }
}

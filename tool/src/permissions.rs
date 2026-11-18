//! What the tool asks of the daemon, declared at its registration.

use diverge_sdk::shared::permission::agents::{Over, Permission, Within};
use diverge_sdk::shared::permission::{Grant, Reach, daemons};

use crate::arguments::Arguments;

/// The tool's permissions, computed from its arguments: what reaching
/// each agent listed takes, and nothing else. The rules:
///
/// 1. **An agent on this daemon is reached by `message`.** One grant
///    over agents: `message`, within exactly the agents listed that
///    have no `relay`. Watching a message is part of sending it, and
///    asks for nothing more. Where every agent has a `relay` there is
///    no such grant.
/// 2. **An agent on another daemon is reached by `relay`.** One grant
///    over remote daemons: `relay`, within exactly the records the
///    routes begin at, each named once, in ascending order. A daemon
///    that is handed a request with a `relay` judges that the caller
///    may relay to the first record and nothing else of the request,
///    so the agent is in no grant here: its template and index are
///    the far daemon's, which judges the message as the account the
///    record names. Where no agent has a `relay` there is no such
///    grant.
/// 3. **`relay` is the whole of what a record allows.** It is one
///    action for every use of a remote daemon, so the tool may relay
///    any request to the records it lists, and not messages only. What
///    comes of one is the far daemon's to judge.
pub fn permissions(arguments: &Arguments) -> Vec<Grant> {
    let mut grants = Vec::new();
    // Rule 1.
    let here: Vec<_> = arguments.agents.iter().filter(|agent| agent.relay.is_none()).map(|agent| agent.key()).collect();
    if !here.is_empty() {
        grants.push(Grant::Agents(Permission::Over {
            actions: vec![Over::Message],
            within: Within::Agents(here),
        }));
    }
    // Rule 2.
    let mut ids: Vec<_> = arguments.agents.iter().filter_map(|agent| agent.relay.as_ref()?.first().copied()).collect();
    ids.sort();
    ids.dedup();
    if !ids.is_empty() {
        grants.push(Grant::Daemons(daemons::Permission::Over {
            actions: vec![daemons::Over::Relay],
            within: Reach::Listed(daemons::Listed { ids }),
        }));
    }
    grants
}

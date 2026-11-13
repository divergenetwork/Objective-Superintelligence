//! What the tool asks of the daemon, declared at its registration.

use diverge_sdk::shared::permission::Grant;
use diverge_sdk::shared::permission::agents::{Over, Permission, Within};

use crate::arguments::Arguments;

/// The tool's permissions: `message` over exactly the agents its
/// arguments list, which is how a decision reaches them. Watching a
/// message is part of sending it, and asks for nothing more. Nothing
/// else.
pub fn permissions(arguments: &Arguments) -> Vec<Grant> {
    vec![Grant::Agents(Permission::Over {
        actions: vec![Over::Message],
        within: Within::Agents(arguments.agents.iter().map(|agent| agent.key()).collect()),
    })]
}

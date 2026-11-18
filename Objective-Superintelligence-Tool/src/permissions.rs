//! What the tool asks of the daemon, declared at its registration.

use diverge_sdk::shared::permission::agents::{Over, Permission, Within};
use diverge_sdk::shared::permission::{Grant, Reach, daemons};

/// The tool's permissions: what reaching any agent a call may list
/// takes, and nothing else. The rules:
///
/// 1. **Nothing is known at registration.** The tool takes no
///    arguments, and the agents a decision is put to come with each
///    call of `structured_decisions`. So nothing can be listed, and
///    each grant reaches `"any"`.
/// 2. **An agent on this daemon is reached by `message`**, over
///    agents. Watching a message is part of sending it, and asks for
///    nothing more.
/// 3. **An agent on another daemon is reached by `relay`**, over
///    remote daemons. A daemon handed a request with a `relay` judges
///    that the caller may relay to the first record and nothing else
///    of the request; the daemon the route ends at judges the message,
///    as the account the record names. `relay` is one action for every
///    use of a remote daemon, so it allows any request relayed there,
///    not messages only.
pub fn permissions() -> Vec<Grant> {
    vec![
        // Rule 2.
        Grant::Agents(Permission::Over {
            actions: vec![Over::Message],
            within: Within::Any,
        }),
        // Rule 3.
        Grant::Daemons(daemons::Permission::Over {
            actions: vec![daemons::Over::Relay],
            within: Reach::Any,
        }),
    ]
}

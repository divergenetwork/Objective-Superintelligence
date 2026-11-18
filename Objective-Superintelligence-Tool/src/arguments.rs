//! What the tool takes as its arguments: nothing.

use serde::{Deserialize, Serialize};

/// The arguments the tool is made with.
///
/// None: the tool is the same for every agent that declares it, and
/// what it serves is decided by who calls, not by how it was made.
/// `{}` is the one value; any member is refused, so a misspelled
/// setting is never silently ignored.
///
/// # How it runs
///
/// As a dependency tool. What it serves depends on its caller, as the
/// daemon attests it on every request: an
/// Objective-Superintelligence agent is served `structured_decisions`,
/// which puts a decision to the agents the call lists; every other
/// caller is served `sum`. See [`caller`](crate::caller).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Arguments {}

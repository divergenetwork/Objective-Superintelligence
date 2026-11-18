//! The one tool the agent depends on, as it declares it.

use diverge_sdk::shared::containers::dependencies::{Mode, Scope, Template};
use diverge_sdk::shared::containers::request::Image;
use serde_json::json;

/// The memory the tool's container is given, in bytes: 256 MiB.
const MEMORY: u64 = 268_435_456;

/// The disk the tool's container is given, in bytes: 256 MiB.
const DISK: u64 = 268_435_456;

/// The agent's one dependency: the Objective-Superintelligence tool,
/// of the image the agent's arguments name as
/// [`tool`](crate::arguments::Arguments::tool), made with no
/// arguments, `{}`. The agents it asks come with each call, from this
/// agent's own arguments, by
/// [`Arguments::called`](crate::arguments::Arguments::called).
///
/// Lazy and global, with no mount and no secret: one tool for every
/// agent that declares it, whatever its template, run while it is
/// used and stopped when it is not. It keeps nothing, so nothing is
/// lost when it stops, and it may mount nothing, since a mount is
/// only for a tool of one agent.
pub fn dependency(image: Image) -> Template {
    Template {
        image,
        memory: MEMORY,
        disk: DISK,
        arguments: json!({}),
        secrets: Vec::new(),
        mode: Mode::Lazy {
            scope: Scope::Global,
            fuse_file_mounts: Vec::new(),
            fuse_directory_mounts: Vec::new(),
        },
    }
}

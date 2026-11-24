//! The one tool the agent depends on, as it declares it.

use diverge_sdk::shared::containers::dependencies::{Mode, Scope, Template};
use diverge_sdk::shared::containers::request::Image;
use serde_json::json;

/// The image of the Objective-Superintelligence tool this agent was built
/// against: its digest, compiled in from `tool-image.json` beside the
/// crate. No registry is named: the image is one the daemon holds in
/// its Store on a provider, built there with `images build`, and is
/// found by its digest. A new tool image is a new digest there, and a
/// new build of this agent.
///
/// The tool's image holds nothing of this one, so the tool is built
/// first. It is told this agent's image as an argument instead: see
/// [`dependency`].
const TOOL_IMAGE: &str = include_str!("../tool-image.json");

/// The memory the tool's container is given, in bytes: 256 MiB.
const MEMORY: u64 = 268_435_456;

/// The disk the tool's container is given, in bytes: 256 MiB.
const DISK: u64 = 268_435_456;

/// The agent's one dependency: the Objective-Superintelligence tool,
/// of the image compiled in, made with one argument,
/// `superintelligence_agent_digest`: the digest of `own`, the image
/// this agent runs, as its registration states it. The tool serves
/// `structured_decisions` to an agent of that image, and so to this
/// one. The digest alone is given, and none of the image's
/// references, so that one agent image under any names declares one
/// tool. The agents the tool asks come with each call, from this
/// agent's own arguments, by
/// [`Arguments::called`](crate::arguments::Arguments::called).
///
/// Lazy and global, with no mount and no secret: one tool for every
/// agent that declares it with the same arguments, whatever its
/// template, run while it is used and stopped when it is not. It keeps nothing, so nothing is
/// lost when it stops, and it may mount nothing, since a mount is
/// only for a tool of one agent.
pub fn dependency(own: &Image) -> Result<Template, String> {
    let image: Image = serde_json::from_str(TOOL_IMAGE).map_err(|error| format!("the tool image compiled into this agent is not an image: {error}"))?;
    Ok(Template {
        image,
        memory: MEMORY,
        disk: DISK,
        arguments: json!({ "superintelligence_agent_digest": own.digest }),
        secrets: Vec::new(),
        mode: Mode::Lazy {
            scope: Scope::Global,
            fuse_file_mounts: Vec::new(),
            fuse_directory_mounts: Vec::new(),
        },
    })
}

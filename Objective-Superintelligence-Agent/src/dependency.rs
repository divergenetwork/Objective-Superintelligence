//! The one tool the agent depends on, as it declares it.

use diverge_sdk::shared::containers::dependencies::{Mode, Template};
use diverge_sdk::shared::containers::request::Image;
use serde_json::{Value, json};

use crate::arguments::Arguments;

/// The image of the Objective-Superintelligence tool this agent was built
/// against: its digest, compiled in from `tool-image.json` beside the
/// crate. No registry is named: the image is one the daemon holds in
/// its Store on a provider, built there with `images build`, and is
/// found by its digest. A new tool image is a new digest there, and a
/// new build of this agent.
const TOOL_IMAGE: &str = include_str!("../tool-image.json");

/// The memory the tool's container is given, in bytes: 256 MiB.
const MEMORY: u64 = 268_435_456;

/// The disk the tool's container is given, in bytes: 256 MiB.
const DISK: u64 = 268_435_456;

/// The agent's one dependency, the same at every registration but for
/// the agents it carries: the Objective-Superintelligence tool, made with
/// this agent's own agents, each with its `relay` where it has one
/// and its weight left out — the tool asks, and weighs nothing. An
/// agent with no `relay` is written with none, so that the tool's
/// arguments have the one spelling its own have. Transient, with no mount and no secret: it
/// keeps nothing, and runs while this agent's decision does.
pub fn dependency(arguments: &Arguments) -> Result<Template, String> {
    let image: Image = serde_json::from_str(TOOL_IMAGE).map_err(|error| format!("the tool image compiled into this agent is not an image: {error}"))?;
    let agents: Vec<Value> = arguments
        .agents
        .iter()
        .map(|agent| match &agent.relay {
            Some(relay) => json!({ "template": agent.template, "index": agent.index, "relay": relay }),
            None => json!({ "template": agent.template, "index": agent.index }),
        })
        .collect();
    Ok(Template {
        image,
        memory: MEMORY,
        disk: DISK,
        arguments: json!({ "agents": agents }),
        secrets: Vec::new(),
        mode: Mode::Transient {
            fuse_file_mounts: Vec::new(),
            fuse_directory_mounts: Vec::new(),
        },
    })
}

//! Who is calling, and so which tools are served.

use diverge_sdk::daemon::key;
use diverge_sdk::shared::mcp::attestation::{self, Caller};
use rmcp::model::{MetaObject, RequestMetaObject};
use rmcp::service::RequestContext;
use rmcp::RoleServer;

/// The image digest of the Objective-Superintelligence agent.
///
/// A placeholder until the agent's image is built: no image has this
/// digest, so until it is replaced every caller is served [`Mode::Other`].
pub const SUPERINTELLIGENCE_AGENT: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

/// What the tool serves a caller: one of two sets of tools, and never
/// both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// An Objective-Superintelligence agent: `structured_decisions`
    /// alone.
    Superintelligence,
    /// Every other caller: `get` and `vote`. With it, the agent
    /// calling, whose queue the two work on; or none, for a caller
    /// that is not an agent the daemon attested, which has no queue.
    Other(Option<key::Agent>),
}

/// The mode a request is served in, from the caller the daemon attested
/// on it. The rules:
///
/// 1. **Only the attestation counts.** The caller is read from the
///    daemon's stamp under its key of the request's `_meta`, never
///    from anything the request says of itself.
/// 2. **An Objective-Superintelligence agent is told by its image.**
///    An agent whose image digest is [`SUPERINTELLIGENCE_AGENT`] is
///    [`Mode::Superintelligence`], whatever its template, index or
///    name.
/// 3. **Every other agent is [`Mode::Other`], with its key**: its
///    template and index, which name its queue.
/// 4. **Everyone else is [`Mode::Other`], with no key**: a client over
///    a connect, and a request with no attestation or one that does
///    not read as a caller.
pub fn mode(context: &RequestContext<RoleServer>, own: Option<&RequestMetaObject>) -> Mode {
    match attestation::request(meta(context, own)) {
        // Rule 2.
        Ok(Caller::Agent(agent)) if agent.image.digest == SUPERINTELLIGENCE_AGENT => Mode::Superintelligence,
        // Rule 3.
        Ok(Caller::Agent(agent)) => Mode::Other(Some(agent.key)),
        // Rule 4.
        _ => Mode::Other(None),
    }
}

/// The `_meta` the attestation is read from. rmcp moves a request's
/// `_meta` into the context before the handler runs; the params' own
/// is read only when the context holds no attestation and the params
/// do.
fn meta<'a>(context: &'a RequestContext<RoleServer>, own: Option<&'a RequestMetaObject>) -> &'a MetaObject {
    let attested = |meta: &MetaObject| meta.contains_key(diverge_sdk::shared::mcp::KEY);
    match own {
        Some(own) if !attested(&context.meta.0) && attested(&own.0) => &own.0,
        _ => &context.meta.0,
    }
}

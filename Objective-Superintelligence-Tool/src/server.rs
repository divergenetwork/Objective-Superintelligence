//! The MCP server: the tool's state, and the tools served by caller.

use std::sync::Arc;

use rmcp::model::{CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation, ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler};
use schemars::JsonSchema;
use serde_json::Value;

use crate::caller::{Mode, mode};
use crate::daemon::Daemon;
use crate::failure::Failure;
use crate::store::Store;
use crate::voters::Waiters;
use crate::{get, structured_decisions, vote};

/// What every MCP session shares: the daemon connection and the
/// database pool, each made on first use, and the votes awaited.
#[derive(Default)]
pub struct State {
    /// The daemon, and the proxy it is reached through.
    pub daemon: Daemon,
    /// The voters' queues.
    pub store: Store,
    /// The votes the calls in progress are waiting for.
    pub waiters: Waiters,
}

/// The server one MCP session is served by: every session the proxy
/// opens is a clone of one, so the state is the container's.
#[derive(Clone, Default)]
pub struct Server {
    state: Arc<State>,
}

/// The instructions a caller's MCP client is handed on connecting:
/// both modes', since the client is handed them before any request is
/// attested.
const INSTRUCTIONS: &str = "An Objective-Superintelligence agent is served `structured_decisions`: call it with `agents`, \
`content` and `questions`, and `results` comes back with one entry per agent, in order: that agent's answers, or the string \
`failure`. Every other agent is served `get` and `vote`. When you are told a structured decision is in your queue, call `get` \
to read it, then `vote` with its `id` and one answer for each of its questions.";

/// A tool, its input schema the JSON Schema of `P`.
fn tool<P: JsonSchema>(name: &'static str, description: &'static str) -> Tool {
    let schema = serde_json::to_value(schemars::schema_for!(P)).ok().and_then(|value| value.as_object().cloned()).unwrap_or_default();
    Tool::new(name, description, Arc::new(schema))
}

/// The tools a mode serves.
fn served(mode: &Mode) -> Vec<Tool> {
    match mode {
        Mode::Superintelligence => vec![tool::<structured_decisions::Parameters>(structured_decisions::NAME, structured_decisions::DESCRIPTION)],
        Mode::Other(_) => vec![tool::<get::Parameters>(get::NAME, get::DESCRIPTION), tool::<vote::Parameters>(vote::NAME, vote::DESCRIPTION)],
    }
}

/// The call's parameters, as the tool's `P`.
fn parameters<P: serde::de::DeserializeOwned>(request: CallToolRequestParams) -> Result<P, Failure> {
    serde_json::from_value(Value::Object(request.arguments.unwrap_or_default())).map_err(|error| Failure::Parameters(error.to_string()))
}

/// What a call answers with: a structured result, or the parts of a
/// message.
enum Answer {
    /// A structured result.
    Structured(Value),
    /// A message, part by part.
    Parts(Vec<ContentBlock>),
}

/// A tool's answer as a structured result.
fn structured<T: serde::Serialize>(answer: T) -> Result<Answer, Failure> {
    serde_json::to_value(answer).map(Answer::Structured).map_err(|error| Failure::Parameters(error.to_string()))
}

/// The call, in the caller's mode, to its answer. `get` and `vote`
/// are an agent's: a caller with no key has no queue.
async fn call(state: &State, mode: Mode, request: CallToolRequestParams) -> Result<Answer, Failure> {
    match mode {
        Mode::Superintelligence => structured(structured_decisions::call(state, parameters(request)?).await?),
        Mode::Other(None) => Err(Failure::NoQueue),
        Mode::Other(Some(caller)) if request.name == get::NAME => {
            let get::Parameters {} = parameters(request)?;
            Ok(Answer::Parts(get::call(state, &caller).await?))
        }
        Mode::Other(Some(caller)) => structured(vote::call(state, &caller, parameters(request)?).await?),
    }
}

impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("objective-superintelligence", env!("CARGO_PKG_VERSION")))
            .with_instructions(INSTRUCTIONS)
    }

    /// The tools of the caller's mode: see [`mode`].
    async fn list_tools(&self, request: Option<PaginatedRequestParams>, context: RequestContext<RoleServer>) -> Result<ListToolsResult, ErrorData> {
        let mode = mode(&context, request.as_ref().and_then(|request| request.meta.as_ref()));
        Ok(ListToolsResult::with_all_items(served(&mode)))
    }

    /// One of the caller's mode's tools, by name. The rules:
    ///
    /// 1. **A caller is served its mode's tools and no other.** A name
    ///    that is none of them is a protocol error, as a name the tool
    ///    has nowhere is: `get` and `vote` are no tools to an
    ///    Objective-Superintelligence agent, and `structured_decisions`
    ///    is none to any other caller.
    /// 2. **The call runs on a task of its own**, and is waited for.
    ///    A call that has put a decision in a queue, or taken an entry
    ///    out of one, has more to do before it is done; a request that
    ///    is dropped part way must not leave that undone.
    /// 3. **The call's own failure is its error result**, for the
    ///    caller to read.
    async fn call_tool(&self, request: CallToolRequestParams, context: RequestContext<RoleServer>) -> Result<CallToolResponse, ErrorData> {
        let mode = mode(&context, request.meta.as_ref());
        // Rule 1.
        if !served(&mode).iter().any(|tool| tool.name == request.name) {
            return Err(ErrorData::invalid_params(format!("the tool has no tool named {}", request.name), None));
        }
        // Rule 2.
        let state = Arc::clone(&self.state);
        let outcome = match tokio::spawn(async move { call(&state, mode, request).await }).await {
            Ok(outcome) => outcome,
            Err(error) => Err(Failure::Ended(error.to_string())),
        };
        // Rule 3.
        Ok(CallToolResponse::from(match outcome {
            Ok(Answer::Structured(value)) => CallToolResult::structured(value),
            Ok(Answer::Parts(parts)) => CallToolResult::success(parts),
            Err(failure) => CallToolResult::error(vec![ContentBlock::text(failure.to_string())]),
        }))
    }
}

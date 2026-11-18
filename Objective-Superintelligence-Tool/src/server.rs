//! The MCP server: the tool's state, and the tools served by caller.

use std::sync::Arc;

use rmcp::model::{CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation, ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler};
use schemars::JsonSchema;
use serde_json::Value;

use crate::caller::{Mode, mode};
use crate::daemon::Daemon;
use crate::failure::Failure;
use crate::{structured_decisions, sum};

/// The server one MCP session is served by: every session the proxy
/// opens is a clone of one, so the daemon connection is the
/// container's.
#[derive(Clone, Default)]
pub struct Server {
    daemon: Arc<Daemon>,
}

/// The instructions a caller's MCP client is handed on connecting:
/// both modes', since the client is handed them before any request is
/// attested.
const INSTRUCTIONS: &str = "An Objective-Superintelligence agent is served `structured_decisions`: call it with `agents`, \
`content` and `questions`, and `results` comes back with one entry per agent, in order: that agent's answers, or the string \
`failure`. Every other caller is served `sum`: call it with `a` and `b`.";

/// A tool, its input schema the JSON Schema of `P`.
fn tool<P: JsonSchema>(name: &'static str, description: &'static str) -> Tool {
    let schema = serde_json::to_value(schemars::schema_for!(P)).ok().and_then(|value| value.as_object().cloned()).unwrap_or_default();
    Tool::new(name, description, Arc::new(schema))
}

/// The one tool a mode serves, and its name.
fn served(mode: Mode) -> Tool {
    match mode {
        Mode::Superintelligence => tool::<structured_decisions::Parameters>(structured_decisions::NAME, structured_decisions::DESCRIPTION),
        Mode::Other => tool::<sum::Parameters>(sum::NAME, sum::DESCRIPTION),
    }
}

/// The call's parameters, as the tool's `P`.
fn parameters<P: serde::de::DeserializeOwned>(request: CallToolRequestParams) -> Result<P, Failure> {
    serde_json::from_value(Value::Object(request.arguments.unwrap_or_default())).map_err(|error| Failure::Parameters(error.to_string()))
}

impl Server {
    /// The call, in the caller's mode, to its structured answer.
    async fn call(&self, mode: Mode, request: CallToolRequestParams) -> Result<Value, Failure> {
        let answer = match mode {
            Mode::Superintelligence => serde_json::to_value(structured_decisions::call(&self.daemon, parameters(request)?).await?),
            Mode::Other => serde_json::to_value(sum::call(parameters(request)?)),
        };
        answer.map_err(|error| Failure::Parameters(error.to_string()))
    }
}

impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("objective-superintelligence", env!("CARGO_PKG_VERSION")))
            .with_instructions(INSTRUCTIONS)
    }

    /// The one tool of the caller's mode: see [`mode`].
    async fn list_tools(&self, request: Option<PaginatedRequestParams>, context: RequestContext<RoleServer>) -> Result<ListToolsResult, ErrorData> {
        let mode = mode(&context, request.as_ref().and_then(|request| request.meta.as_ref()));
        Ok(ListToolsResult::with_all_items(vec![served(mode)]))
    }

    /// The caller's mode's one tool, by name. The rules:
    ///
    /// 1. **A caller is served its mode's tool and no other.** A name
    ///    that is not that tool's is a protocol error, as a name the
    ///    tool has nowhere is: `sum` is no tool to an
    ///    Objective-Superintelligence agent, and `structured_decisions`
    ///    none to any other caller.
    /// 2. **The call's own failure is its error result**, for the
    ///    caller to read.
    async fn call_tool(&self, request: CallToolRequestParams, context: RequestContext<RoleServer>) -> Result<CallToolResponse, ErrorData> {
        let mode = mode(&context, request.meta.as_ref());
        // Rule 1.
        if request.name != served(mode).name {
            return Err(ErrorData::invalid_params(format!("the tool has no tool named {}", request.name), None));
        }
        // Rule 2.
        Ok(CallToolResponse::from(match self.call(mode, request).await {
            Ok(value) => CallToolResult::structured(value),
            Err(failure) => CallToolResult::error(vec![ContentBlock::text(failure.to_string())]),
        }))
    }
}

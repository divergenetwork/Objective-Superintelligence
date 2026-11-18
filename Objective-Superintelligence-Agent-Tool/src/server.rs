//! The MCP server: the tool's state, and the one tool served.

use std::sync::Arc;

use rmcp::model::{CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation, ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler};
use serde_json::Value;

use crate::daemon::Daemon;
use crate::failure::Failure;
use crate::structured_decisions;

/// The server one MCP session is served by: every session the proxy
/// opens is a clone of one, so the daemon connection is the
/// container's.
#[derive(Clone, Default)]
pub struct Server {
    daemon: Arc<Daemon>,
}

/// The instructions a caller's MCP client is handed on connecting.
const INSTRUCTIONS: &str = "Puts one structured decision to a fixed list of agents. Call `structured_decisions` with `content` and \
`questions`; `results` comes back with one entry per agent, in order: that agent's answers, or the string `failure`.";

impl Server {
    /// The one tool, its input schema the JSON Schema of its
    /// parameters.
    fn tool() -> Tool {
        let schema = serde_json::to_value(schemars::schema_for!(structured_decisions::Parameters)).ok().and_then(|value| value.as_object().cloned()).unwrap_or_default();
        Tool::new(structured_decisions::NAME, structured_decisions::DESCRIPTION, Arc::new(schema))
    }

    /// The call, to its structured answer.
    async fn call(&self, request: CallToolRequestParams) -> Result<Value, Failure> {
        let parameters = serde_json::from_value(Value::Object(request.arguments.unwrap_or_default())).map_err(|error| Failure::Parameters(error.to_string()))?;
        let results = structured_decisions::call(&self.daemon, parameters).await?;
        serde_json::to_value(results).map_err(|error| Failure::Parameters(error.to_string()))
    }
}

impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("objective-superintelligence", env!("CARGO_PKG_VERSION")))
            .with_instructions(INSTRUCTIONS)
    }

    async fn list_tools(&self, _request: Option<PaginatedRequestParams>, _context: RequestContext<RoleServer>) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(vec![Server::tool()]))
    }

    /// The one tool by name. The call's own failure is its error
    /// result, for the caller to read; only a name the tool does not
    /// have is a protocol error. Whoever the daemon let call is
    /// served: the tool asks the agents it was made with, whoever
    /// asks it.
    async fn call_tool(&self, request: CallToolRequestParams, _context: RequestContext<RoleServer>) -> Result<CallToolResponse, ErrorData> {
        if request.name != structured_decisions::NAME {
            return Err(ErrorData::invalid_params(format!("the tool has no tool named {}", request.name), None));
        }
        Ok(CallToolResponse::from(match self.call(request).await {
            Ok(value) => CallToolResult::structured(value),
            Err(failure) => CallToolResult::error(vec![ContentBlock::text(failure.to_string())]),
        }))
    }
}

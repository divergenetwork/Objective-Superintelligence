//! The dependency tool, as the agent calls it.

use diverge_sdk::container_proxy::inside::Client;
use diverge_sdk::shared::containers::structured_decisions::Decision;
use rmcp::model::{CallToolRequestParams, Tool};
use serde_json::{Value, json};

use crate::failure::Failure;
use crate::outcome::{Outcome, Results};

/// The tool's own name, which the daemon lists beneath a prefix:
/// `<prefix>_structured_decisions`.
const NAME: &str = "structured_decisions";

/// Put the decision to the agents, through the dependency tool: its
/// outcomes, one per agent, in the agents' order.
///
/// The call is an ordinary MCP tool call through the proxy beside the
/// program, as any agent calls any of its tools, and names the agents
/// to ask, as [`Arguments::called`](crate::arguments::Arguments::called)
/// writes them. There must be one outcome per agent; a result of
/// another length is not the tool's, and is refused whole rather than
/// matched to the wrong weights.
pub async fn ask(client: &Client, decision: &Decision, agents: &[Value]) -> Result<Vec<Outcome>, Failure> {
    let name = name(client).await?;
    let arguments = json!({ "agents": agents, "content": decision.content, "questions": decision.questions });
    let Value::Object(arguments) = arguments else {
        return Err(Failure::Result("the call's arguments are not an object".to_owned()));
    };
    let mut params = CallToolRequestParams::new(name);
    params.arguments = Some(arguments);
    let result = client.mcp_call_tool(params).await.map_err(|error| Failure::Mcp(error.to_string()))?;
    // The result is read as JSON, as it is on the wire, so that what
    // is read is what the tool wrote and nothing of a client's own
    // reading of it.
    let result = serde_json::to_value(&result).map_err(|error| Failure::Result(error.to_string()))?;
    if result.get("isError").and_then(Value::as_bool) == Some(true) {
        let why = result
            .get("content")
            .and_then(Value::as_array)
            .and_then(|content| content.iter().find_map(|block| block.get("text").and_then(Value::as_str)))
            .unwrap_or("it gave no reason");
        return Err(Failure::Tool(why.to_owned()));
    }
    let structured = result.get("structuredContent").cloned().ok_or_else(|| Failure::Result("it carries no structured content".to_owned()))?;
    let results: Results = serde_json::from_value(structured).map_err(|error| Failure::Result(error.to_string()))?;
    if results.results.len() != agents.len() {
        return Err(Failure::Result(format!("it holds {} outcomes for {} agents", results.results.len(), agents.len())));
    }
    Ok(results.results)
}

/// The dependency tool's name among the agent's tools.
///
/// The rules:
///
/// 1. A tool is a candidate when its name is a prefix, one `_`, and
///    `structured_decisions`: the daemon lists every tool as
///    `<prefix>_<the tool's own name>`.
/// 2. Among the candidates, one the daemon marks a dependency is the
///    tool; that is this agent's own, since it declares exactly one.
/// 3. With no candidate marked so, the first candidate is the tool.
/// 4. With no candidate, the agent has no such tool.
async fn name(client: &Client) -> Result<String, Failure> {
    let peer = client.mcp_peer().await.map_err(|error| Failure::Mcp(error.to_string()))?;
    let tools = peer.list_all_tools().await.map_err(|error| Failure::Mcp(error.to_string()))?;
    let candidates: Vec<&Tool> = tools.iter().filter(|tool| tool.name.split_once('_').is_some_and(|(_, own)| own == NAME)).collect();
    let tool = candidates.iter().find(|tool| is_dependency(tool)).or(candidates.first()).ok_or(Failure::NoTool)?;
    Ok(tool.name.to_string())
}

/// Whether the daemon marks the tool a dependency of the agent's: the
/// `kind` it states under its own key of the tool's `_meta`.
fn is_dependency(tool: &Tool) -> bool {
    serde_json::to_value(tool)
        .ok()
        .and_then(|tool| tool.get("_meta")?.get(diverge_sdk::shared::mcp::KEY)?.get("kind")?.as_str().map(|kind| kind == "dependency"))
        .unwrap_or(false)
}

//! `tool`, as a tool container: what it serves depends on who calls.
//!
//! The tool takes no arguments. Every request it receives carries the
//! caller the daemon attested, and the tool serves one of two modes by
//! it — see [`caller`]:
//!
//! - **An Objective-Superintelligence agent** is served
//!   `structured_decisions`, which takes the agents to ask, each by
//!   its template and index and, for one on another daemon, the route
//!   to it, with a decision — a prompt and typed questions — and
//!   sends the decision to every one of them through the daemon,
//!   watching each, and returns what each answered, or that it failed.
//!   It weighs nothing and averages nothing: that is the agent's work.
//! - **Every other caller** is served `sum`, which adds two numbers.
//!
//! The program is an HTTP server on the container's loopback, at the
//! port [`diverge_sdk::container_proxy::inside::port()`] names, and
//! the proxy beside it is its only caller: `POST /register` once,
//! `POST /secrets` and `POST /secrets/collect` around every activation
//! (the tool takes no secret), and the MCP server at `/mcp`.
//! `--schema`, the program's one argument, prints the JSON Schema of
//! the arguments it takes, which is the `schema` of the image's
//! `/.diverge/DIVERGE.md` frontmatter.
//!
//! # Nothing of the proxy's is counted on at the start
//!
//! The host injects the proxy at runtime, and it may not be up when
//! the program starts. So the daemon is reached when a call first
//! needs it, and kept.

mod agent;
mod arguments;
mod ask;
mod caller;
mod daemon;
mod failure;
mod outcome;
mod permissions;
mod register;
mod secrets;
mod server;
mod structured_decisions;
mod sum;

use std::io::Write as _;
use std::sync::Arc;

use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::{StreamableHttpServerConfig, StreamableHttpService};

use crate::arguments::Arguments;
use crate::server::Server;

/// The program: the server, or — on `--schema` — the schema.
///
/// `--schema` alone prints the JSON Schema of [`Arguments`] on stdout
/// and exits 0; no argument is the server, for the container's life;
/// anything else is a usage line on stderr and exit 2. The image's
/// entrypoint never passes the argument: it exists so `/.diverge/DIVERGE.md`
/// is written from the type the registration reads.
fn main() {
    let mut arguments = std::env::args_os().skip(1);
    match (arguments.next(), arguments.next()) {
        (None, None) => {}
        (Some(argument), None) if argument == "--schema" => return schema(),
        _ => {
            let _ = writeln!(std::io::stderr(), "usage: tool [--schema]");
            std::process::exit(2);
        }
    }
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("the runtime could not be built");
    runtime.block_on(serve());
}

/// `--schema`: the JSON Schema of [`Arguments`], pretty-printed on
/// stdout with a trailing newline; a failure to write it is said on
/// stderr and exit 1, never a partial schema passed off as the answer.
fn schema() {
    let text = match serde_json::to_string_pretty(&schemars::schema_for!(Arguments)) {
        Ok(text) => text,
        Err(error) => {
            let _ = writeln!(std::io::stderr(), "{error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = writeln!(std::io::stdout(), "{text}") {
        let _ = writeln!(std::io::stderr(), "{error}");
        std::process::exit(1);
    }
}

/// The server, for the container's life, bound on the loopback: the
/// proxy is the only thing that dials it, and it is beside it. Every
/// MCP session the proxy opens shares one [`Server`], so the daemon
/// connection is made once.
async fn serve() {
    let server = Server::default();
    let mcp = StreamableHttpService::new(move || Ok(server.clone()), Arc::new(LocalSessionManager::default()), StreamableHttpServerConfig::default());
    let app = axum::Router::new()
        .route("/register", axum::routing::post(register::register))
        .route("/secrets", axum::routing::post(secrets::feed))
        .route("/secrets/collect", axum::routing::post(secrets::collect))
        .nest_service("/mcp", mcp);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", diverge_sdk::container_proxy::inside::port())).await.expect("the port could not be bound");
    axum::serve(listener, app).await.expect("the server stopped unexpectedly");
}

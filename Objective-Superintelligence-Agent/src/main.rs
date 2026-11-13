//! `objective-superintelligence-agent`, as an agent container: a
//! structured decision, answered with the weighted average of other
//! agents' answers.
//!
//! The agent is made with a list of agents, each with a weight. It
//! takes structured decisions and nothing else. For each one it calls
//! its one dependency tool, Objective-Superintelligence-Tool, which
//! puts the same decision to every one of those agents and returns
//! what each answered; the agent then answers every question with the
//! average of their answers, each weighed by its agent's weight.
//!
//! The program is an HTTP server on the container's loopback, at the
//! port [`diverge_sdk::container_proxy::inside::port()`] names, and
//! the proxy beside it is its only caller: `POST /register` once,
//! `POST /secrets` and `POST /secrets/collect` around every activation
//! (the agent takes no secret), `POST /run`, which is one decision,
//! and `POST /enqueue`, which is chat and is refused. `--schema`, the
//! program's one argument, prints the JSON Schema of the arguments it
//! takes, which is the `schema` of the image's `/DIVERGE.md`
//! frontmatter.

mod arguments;
mod average;
mod dependency;
mod failure;
mod outcome;
mod register;
mod run;
mod secrets;
mod tool;

use std::io::Write as _;

use diverge_sdk::container_proxy::inside::Client;

use crate::arguments::Arguments;

/// The program: the server, or — on `--schema` — the schema.
///
/// `--schema` alone prints the JSON Schema of [`Arguments`] on stdout
/// and exits 0; no argument is the server, for the container's life;
/// anything else is a usage line on stderr and exit 2. The image's
/// entrypoint never passes the argument: it exists so `/DIVERGE.md`
/// is written from the type the registration reads.
fn main() {
    let mut arguments = std::env::args_os().skip(1);
    match (arguments.next(), arguments.next()) {
        (None, None) => {}
        (Some(argument), None) if argument == "--schema" => return schema(),
        _ => {
            let _ = writeln!(std::io::stderr(), "usage: objective-superintelligence-agent [--schema]");
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
/// run shares one [`Client`], so the MCP session with the proxy is
/// made once, when a run first needs it.
async fn serve() {
    let client = std::sync::Arc::new(Client::new());
    let app = axum::Router::new()
        .route("/register", axum::routing::post(register::register))
        .route("/secrets", axum::routing::post(secrets::feed))
        .route("/secrets/collect", axum::routing::post(secrets::collect))
        .route("/run", axum::routing::post(run::run))
        .route("/enqueue", axum::routing::post(run::enqueue))
        .with_state(client);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", diverge_sdk::container_proxy::inside::port())).await.expect("the port could not be bound");
    axum::serve(listener, app).await.expect("the server stopped unexpectedly");
}

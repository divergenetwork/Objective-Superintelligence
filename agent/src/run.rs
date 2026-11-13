//! `POST /run` and `POST /enqueue`: one decision, and no chat.
//!
//! The rules:
//!
//! 1. A chat run is `400`: the agent's image declares no chat, so the
//!    daemon never sends one, and the program takes none. An enqueue
//!    is chat by definition, and is `400` the same.
//! 2. A decision before the registration is `409`, `unregistered`.
//! 3. Otherwise the run is `200`, server-sent events, each chunk one
//!    event's data:
//!    1. the decision, echoed as it was handed over, under its key —
//!       first, before anything is asked of anyone, so the daemon
//!       logs it and whoever watches it sees it taken up;
//!    2. the answers, one `structured_decisions_response`: every
//!       question's weighted average, as [`average`] states it;
//!    3. the end of the stream.
//! 4. A decision that cannot be answered — the tool unreachable or
//!    failed, or no agent with an answer that fits — has, in place of
//!    step 3.2, one fatal notification saying why. No answer is made
//!    up.
//! 5. Nothing is timed: the run lasts as long as the agents asked
//!    take. A comment goes out on the stream when it is quiet, so the
//!    connection is never idle for long; the proxy reads only data.

use std::convert::Infallible;
use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use diverge_sdk::container_proxy::inside::Client;
use diverge_sdk::container_proxy::inside::agent::{enqueue, run};
use diverge_sdk::provider::endpoints::containers::agents::run::server::response::{AgenticLoopChunk, NotificationChunk, StructuredDecisionsResponseChunk, UserStructuredDecisionsChunk};
use diverge_sdk::shared::containers::structured_decisions::Decision;
use futures_util::Stream;
use rust_decimal::Decimal;
use serde_json::{Value, json};

use crate::average::average;
use crate::failure::Failure;
use crate::register;
use crate::tool;

/// A refusal: the status, and a JSON reason.
type Refusal = (StatusCode, Json<Value>);

/// What a chat is refused with: rule 1.
fn no_chat() -> Refusal {
    (StatusCode::BAD_REQUEST, Json(json!({ "kind": "content", "error": "this agent takes structured decisions only, and no chat" })))
}

/// Rules 1 to 5.
pub async fn run(State(client): State<Arc<Client>>, Json(request): Json<run::request::Request>) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Refusal> {
    let decision = match request {
        run::request::Request::StructuredDecisions(decision) => decision,
        run::request::Request::Chat { .. } => return Err(no_chat()),
    };
    let Some(weights) = register::weights() else {
        return Err((StatusCode::CONFLICT, Json(json!({ "kind": "unregistered", "error": "the agent is not registered" }))));
    };
    let events = async_stream::stream! {
        // Rule 3.1.
        let echo = AgenticLoopChunk::UserStructuredDecisions(UserStructuredDecisionsChunk {
            r#type: Default::default(),
            decision: decision.clone(),
            meta: None,
        });
        if let Some(event) = event(&echo) {
            yield Ok(event);
        }
        // Rules 3.2 and 4.
        let last = match decide(&client, &decision, weights).await {
            Ok(decisions) => AgenticLoopChunk::StructuredDecisionsResponse(decisions),
            Err(failure) => AgenticLoopChunk::Notification(NotificationChunk {
                r#type: Default::default(),
                is_fatal: true,
                message: Value::String(failure.to_string()),
                meta: None,
            }),
        };
        if let Some(event) = event(&last) {
            yield Ok(event);
        }
    };
    Ok(Sse::new(events).keep_alive(KeepAlive::default()))
}

/// Rule 1.
pub async fn enqueue(Json(_request): Json<enqueue::request::Request>) -> Refusal {
    no_chat()
}

/// The decision's answers: the agents asked through the tool, and
/// their answers averaged.
async fn decide(client: &Client, decision: &Decision, weights: &[Decimal]) -> Result<StructuredDecisionsResponseChunk, Failure> {
    let outcomes = tool::ask(client, decision, weights.len()).await?;
    Ok(StructuredDecisionsResponseChunk {
        r#type: Default::default(),
        decisions: average(&decision.questions, &outcomes, weights)?,
        meta: None,
    })
}

/// A chunk as one event's data; none for a chunk that cannot be
/// written, which is then not sent at all.
fn event(chunk: &AgenticLoopChunk) -> Option<Event> {
    serde_json::to_string(chunk).ok().map(|data| Event::default().data(data))
}

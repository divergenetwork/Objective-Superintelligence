//! One decision, put to one agent, and what came of it.

use diverge_sdk::daemon::agent_log::Item;
use diverge_sdk::daemon::endpoints::agents::message;
use diverge_sdk::daemon::endpoints::agents::message::client::request::{Frame, Message};
use diverge_sdk::daemon::endpoints::agents::message::server::response;
use diverge_sdk::daemon::endpoints::logs::server::response::Entry;
use diverge_sdk::shared::containers::structured_decisions::Question;
use diverge_sdk::wire::client::handle::Handle;
use futures_util::StreamExt as _;
use rmcp::model::ContentBlock;

use crate::arguments::Agent;
use crate::daemon::Daemon;
use crate::outcome::Outcome;

/// Put the decision to `agent`, watching, and read what comes back to
/// its end.
///
/// A watched message answers with the agent's log for that message,
/// and the rules here follow the rules of that stream:
///
/// 1. **The first item is the echo**: the decision as the agent took
///    it up, carrying the id its answer will carry.
/// 2. **The answer is the response with the echo's id.** It is the
///    last item the daemon sends, and it is the [`Outcome::Answer`].
/// 3. **Anything else that ends the stream is a failure**: a refusal
///    in place of the echo (forbidden, no structured decisions,
///    modalities the agent does not take, an error, `dropped`), a
///    stream that could not be opened or broke, or a stream that ends
///    — as it does at the agent's going idle — with no response of
///    that id. Items between the echo and the answer are the agent's
///    work and are passed over.
///
/// Nothing is timed: the stream ends by itself when the agent's run
/// does.
pub async fn ask(daemon: &Daemon, handle: &Handle, agent: &Agent, content: &[ContentBlock], questions: &[Question]) -> Outcome {
    let request = Frame {
        agent: agent.reference(),
        message: Message::StructuredDecisions {
            content: content.to_vec(),
            questions: questions.to_vec(),
        },
        watch: Some(true),
        relay: None,
    };
    // The cancel is held to the end and never used: letting go of it
    // early must not be what ends the watch.
    let (stream, _cancel) = match message::client::execute::watch(handle, &request).await {
        Ok(opened) => opened,
        Err(_) => {
            daemon.forget().await;
            return Outcome::Failure;
        }
    };
    let mut stream = std::pin::pin!(stream);
    let mut echoed = None;
    while let Some(frame) = stream.next().await {
        let Ok(response::Frame::Item(Entry::Agent(entry))) = frame else {
            return Outcome::Failure;
        };
        match entry.item {
            Item::UserStructuredDecisions(echo) if echoed.is_none() => echoed = Some(echo.id),
            Item::StructuredDecisionsResponse(answer) if echoed == Some(answer.id) => return Outcome::Answer(answer.chunk.decisions),
            _ => {}
        }
    }
    Outcome::Failure
}

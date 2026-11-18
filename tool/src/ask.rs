//! One decision, put to one agent, and what came of it.

use diverge_sdk::daemon::agent_log;
use diverge_sdk::daemon::agent_log::Item;
use diverge_sdk::daemon::endpoints::agents::message;
use diverge_sdk::daemon::endpoints::agents::message::client::request::{Frame, Message};
use diverge_sdk::daemon::endpoints::agents::message::server::response;
use diverge_sdk::daemon::endpoints::logs::server::response::Entry;
use diverge_sdk::shared::containers::structured_decisions;
use diverge_sdk::shared::containers::structured_decisions::Question;
use diverge_sdk::wire::client::handle::Handle;
use futures_util::StreamExt as _;
use rmcp::model::ContentBlock;

use crate::arguments::Agent;
use crate::daemon::Daemon;
use crate::outcome::Outcome;

/// Put the decision to `agent`, watching, and read what comes back to
/// its end. An agent with a `relay` is asked on the daemon its route
/// ends at: the request carries the route, and this daemon and each
/// after it carry the request on and the answers back, so what is
/// read here is that daemon's stream, by the same rules.
///
/// A watched message answers with the agent's log for that message,
/// and the rules here follow the rules of that stream:
///
/// 1. **The first item is the echo**: the decision as the agent took
///    it up, carrying the id its answer will carry.
/// 2. **The answer is the response with the echo's id.** It is the
///    last item the daemon sends, and it is the [`Outcome::Answer`]:
///    each answer as the agent said it, by [`said`].
/// 3. **Anything else that ends the stream is a failure**: a refusal
///    in place of the echo (forbidden, a route that could not be
///    taken, no structured decisions,
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
        relay: agent.relay(),
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
            Item::StructuredDecisionsResponse(answer) if echoed == Some(answer.id) => {
                return Outcome::Answer(answer.decisions.into_iter().map(|(question, kept)| (question, said(kept))).collect());
            }
            _ => {}
        }
    }
    Outcome::Failure
}

/// An answer as the agent's program said it, out of the answer its
/// daemon's log kept: the probabilities, and neither the `choice` nor
/// the `score` the daemon computed from them. What this tool returns
/// is what an agent says, so that whoever averages it averages
/// probabilities alone, and the daemon that logs the average computes
/// its `choice` and `score` as it does any agent's.
fn said(kept: agent_log::Answer) -> structured_decisions::Answer {
    match kept {
        agent_log::Answer::Noul(noul) => structured_decisions::Answer::Noul(noul),
        agent_log::Answer::Choice(choice) => structured_decisions::Answer::Choice(structured_decisions::Choice {
            probabilities: choice.probabilities,
        }),
        agent_log::Answer::Score(score) => structured_decisions::Answer::Score(structured_decisions::Score {
            probabilities: score.probabilities,
        }),
    }
}

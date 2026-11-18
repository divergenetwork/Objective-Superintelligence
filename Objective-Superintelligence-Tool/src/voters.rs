//! One decision, put to one voter, and what came of it.

use std::collections::HashMap;

use diverge_sdk::daemon::endpoints::agents::message;
use diverge_sdk::daemon::endpoints::agents::message::client::request::{Frame, Message};
use diverge_sdk::shared::containers::structured_decisions::Answer;
use diverge_sdk::wire::client::handle::Handle;
use futures_util::StreamExt as _;
use indexmap::IndexMap;
use rmcp::model::ContentBlock;
use sqlx::PgPool;
use tokio::sync::{Mutex, oneshot};

use crate::agent::Voter;
use crate::daemon::Daemon;
use crate::outcome::Outcome;
use crate::store::queue;

/// What a voter is sent when a decision is put in its queue, word for
/// word.
pub const MESSAGE: &str = "<Objective-Superintelligence>\nYou have a new structured decision in the queue.\n</Objective-Superintelligence>";

/// A voter's answers to one decision, each by its question's id.
pub type Answers = IndexMap<String, Answer>;

/// The votes awaited: for each entry a call is waiting on, where its
/// vote is handed. The `vote` tool hands; [`poll`] waits.
#[derive(Default)]
pub struct Waiters {
    held: Mutex<HashMap<u32, oneshot::Sender<Answers>>>,
}

impl Waiters {
    /// Wait for the vote on `entry`: where it will arrive.
    pub async fn wait(&self, entry: u32) -> oneshot::Receiver<Answers> {
        let (hand, vote) = oneshot::channel();
        self.held.lock().await.insert(entry, hand);
        vote
    }

    /// Hand the vote on `entry` to whoever waits for it; whether anyone
    /// did.
    pub async fn hand(&self, entry: u32, answers: Answers) -> bool {
        match self.held.lock().await.remove(&entry) {
            Some(hand) => hand.send(answers).is_ok(),
            None => false,
        }
    }

    /// Wait for the vote on `entry` no longer.
    pub async fn leave(&self, entry: u32) {
        self.held.lock().await.remove(&entry);
    }
}

/// Tell `voter` that a decision waits in its queue, watching, and wait
/// for its vote on `entry` for as long as the watch lasts.
///
/// The voter is sent [`MESSAGE`], a chat, watched. The watch is what
/// gives the voter its time: the stream lasts while the voter works on
/// the message, and ends when its run does. The rules:
///
/// 1. **A vote ends the wait.** The moment the vote on `entry` is
///    handed over, the watch is cancelled and let go, and the outcome
///    is the vote: the voter's answers.
/// 2. **The watch's end, with no vote, is a failure.** The message
///    could not be sent, the daemon refused it, the stream broke, or
///    the voter went idle without voting. The entry is taken out of
///    the voter's queue, so a vote that comes later finds nothing.
/// 3. **A vote and an ending that cross are settled by the entry.**
///    Whoever takes the entry out first wins. Where the watch ended
///    and the entry is already gone, a vote took it, and that vote is
///    waited for and is the outcome.
pub async fn poll(daemon: &Daemon, waiters: &Waiters, handle: &Handle, pool: &PgPool, voter: &Voter, entry: u32, mut vote: oneshot::Receiver<Answers>) -> Outcome {
    let request = Frame {
        agent: voter.reference(),
        message: Message::Chat {
            content: vec![ContentBlock::text(MESSAGE)],
        },
        watch: Some(true),
        relay: None,
    };
    match message::client::execute::watch(handle, &request).await {
        Ok((stream, cancel)) => {
            let mut stream = std::pin::pin!(stream);
            loop {
                tokio::select! {
                    // Rule 1.
                    answers = &mut vote => {
                        let _ = cancel.cancel().await;
                        return answers.map(Outcome::Answer).unwrap_or(Outcome::Failure);
                    }
                    frame = stream.next() => {
                        if !matches!(frame, Some(Ok(_))) {
                            break;
                        }
                    }
                }
            }
        }
        Err(_) => daemon.forget().await,
    }
    let outcome = match queue::claim(pool, entry).await {
        // Rule 3.
        Ok(false) => vote.await.map(Outcome::Answer).unwrap_or(Outcome::Failure),
        // Rule 2. An entry that could not be taken out stays until
        // the call's end deletes its decision.
        Ok(true) | Err(_) => Outcome::Failure,
    };
    waiters.leave(entry).await;
    outcome
}

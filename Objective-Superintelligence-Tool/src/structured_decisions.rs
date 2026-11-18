//! The Objective-Superintelligence agent's one tool: a decision put to
//! every agent.

use diverge_sdk::shared::containers::structured_decisions::{Question, validate};
use diverge_sdk::wire::client::handle::Handle;
use futures_util::future::join_all;
use rmcp::model::ContentBlock;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::oneshot;

use crate::agent::Agent;
use crate::ask::ask;
use crate::failure::Failure;
use crate::outcome::Outcome;
use crate::server::State;
use crate::store::queue;
use crate::voters::{Answers, poll};

/// The tool's name.
pub const NAME: &str = "structured_decisions";

/// The tool's description, as `tools/list` states it.
pub const DESCRIPTION: &str = "Put one structured decision (a prompt and typed questions) to every agent listed in `agents`, and \
return what each answered. `results` has one entry per agent, in the order `agents` lists them: an object with one member \
`answer`, the agent's answers by question id, or the string `failure` for an agent that gave none.";

/// The tool's parameters: the agents asked, and a decision, as
/// `agents message` takes one.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    /// The agents asked, in the order their answers are returned in.
    /// At least one.
    #[schemars(length(min = 1))]
    pub agents: Vec<Agent>,
    /// What is being decided on: the prompt every question is about.
    pub content: Vec<ContentBlock>,
    /// The questions, each with its own id.
    pub questions: Vec<Question>,
}

/// The tool's answer.
#[derive(Debug, Clone, Serialize)]
pub struct Results {
    /// One outcome per agent, in the order of the call's `agents`.
    pub results: Vec<Outcome>,
}

/// A decision in its voters' queues: the pool it is kept through, its
/// id, and for each voter, in the order `agents` lists them, its
/// entry's id and where its vote will arrive.
struct Queued<'a> {
    pool: &'a PgPool,
    decision: i64,
    entries: Vec<(u32, oneshot::Receiver<Answers>)>,
}

/// Put the decision to every agent at once, and gather what each made
/// of it. The agents do not wait on one another, and one's failure is
/// its own outcome and not the call's. The rules:
///
/// 1. **A structured decision agent is asked by a message**: see
///    [`ask`].
/// 2. **A voter is asked through its queue.** The decision is stored
///    once and put in every voter's queue before any voter is told of
///    it; then each is told, and waited for: see [`poll`]. Where the
///    call lists a voter, the questions are held to what a decision's
///    questions must be, since a vote is checked against them.
/// 3. **The decision is deleted when every agent's outcome is in**,
///    and with it any entry still in a queue. A failure to delete it
///    is not the call's: the outcomes are in hand, and what is left is
///    deleted when the pool is next made.
pub async fn call(state: &State, parameters: Parameters) -> Result<Results, Failure> {
    if parameters.agents.is_empty() {
        return Err(Failure::Parameters("`agents` lists no agent, and there must be at least one".to_owned()));
    }
    let handle = state.daemon.handle().await?;
    // Rule 2.
    let (pool, decision, entries) = if parameters.agents.iter().any(|agent| matches!(agent, Agent::Voter(_))) {
        validate(&parameters.questions).map_err(|invalid| Failure::Parameters(invalid.to_string()))?;
        let queued = enqueue(state, &parameters).await?;
        (Some(queued.pool), Some(queued.decision), queued.entries)
    } else {
        (None, None, Vec::new())
    };
    let mut entries = entries.into_iter();
    let asked = parameters.agents.iter().map(|agent| {
        let entry = match agent {
            Agent::Voter(_) => entries.next(),
            Agent::StructuredDecision(_) => None,
        };
        one(state, &handle, pool, agent, entry, &parameters.content, &parameters.questions)
    });
    let results = join_all(asked).await;
    // Rule 3.
    if let (Some(pool), Some(decision)) = (pool, decision) {
        let _ = queue::finish(pool, decision).await;
    }
    Ok(Results { results })
}

/// One agent's outcome, by its kind.
async fn one(state: &State, handle: &Handle, pool: Option<&PgPool>, agent: &Agent, entry: Option<(u32, oneshot::Receiver<Answers>)>, content: &[ContentBlock], questions: &[Question]) -> Outcome {
    match (agent, pool, entry) {
        (Agent::StructuredDecision(decider), _, _) => ask(&state.daemon, handle, decider, content, questions).await,
        (Agent::Voter(voter), Some(pool), Some((entry, vote))) => poll(&state.daemon, &state.waiters, handle, pool, voter, entry, vote).await,
        // A voter with no entry: none is ever made without one.
        (Agent::Voter(_), _, _) => Outcome::Failure,
    }
}

/// Store the decision and put it in every voter's queue, in one
/// transaction. Each entry's vote is waited for from before the
/// transaction commits, so no voter can read an entry and vote on it
/// before the call is listening. A failure leaves nothing: the
/// transaction is not committed, and no vote is waited for.
async fn enqueue<'a>(state: &'a State, parameters: &Parameters) -> Result<Queued<'a>, Failure> {
    let pool = state.store.pool(state.daemon.client()).await?;
    let content = serde_json::to_string(&parameters.content).map_err(|error| Failure::Parameters(error.to_string()))?;
    let questions = serde_json::to_string(&parameters.questions).map_err(|error| Failure::Parameters(error.to_string()))?;
    let mut transaction = pool.begin().await?;
    let decision = queue::decision(&mut transaction, &content, &questions).await?;
    let mut entries = Vec::new();
    let mut fault = None;
    for agent in &parameters.agents {
        let Agent::Voter(voter) = agent else { continue };
        match queue::entry(&mut transaction, decision, &voter.template, voter.index).await {
            Ok(entry) => entries.push((entry, state.waiters.wait(entry).await)),
            Err(error) => {
                fault = Some(error);
                break;
            }
        }
    }
    let fault = match fault {
        Some(error) => Some(error),
        None => transaction.commit().await.err(),
    };
    if let Some(error) = fault {
        for (entry, _) in &entries {
            state.waiters.leave(*entry).await;
        }
        return Err(error.into());
    }
    Ok(Queued { pool, decision, entries })
}

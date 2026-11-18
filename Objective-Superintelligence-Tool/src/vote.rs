//! A voter's second tool: its vote on one decision.

use std::collections::HashMap;

use diverge_sdk::daemon::key;
use diverge_sdk::shared::containers::structured_decisions;
use diverge_sdk::shared::containers::structured_decisions::{Answer, Question};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::failure::Failure;
use crate::server::State;
use crate::store::queue;
use crate::voters::Answers;

/// The tool's name.
pub const NAME: &str = "vote";

/// The tool's description, as `tools/list` states it.
pub const DESCRIPTION: &str = "Vote on one structured decision you took with `get`, by its `id`. `answers` holds exactly one answer \
for every question of the decision, each naming its question by `id`: for a `noul` question, `answer` is `yes` or `no`; for a \
`choice` question, `option` is the `value` of the one option you pick; for a `score` question, `level` is the number of the one \
level you pick, counting the question's `levels` from 0. A decision is voted on once, and only while it is open.";

/// The tool's parameters: the decision, and an answer to each of its
/// questions.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    /// The decision's id, as `get` gave it: eight hex digits.
    pub id: String,
    /// One answer for every question of the decision.
    #[schemars(length(min = 1))]
    pub answers: Vec<Vote>,
}

/// One question's answer, of the kind the question is. Internally
/// tagged by `type`, snake case, as a question is: `"noul"`,
/// `"choice"`, `"score"`. Each picks exactly one thing: no vote is
/// split, and nothing is ranked.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Vote {
    /// A noul question's answer: yes or no.
    Noul {
        /// The id of the question answered.
        id: String,
        /// Yes or no.
        answer: Said,
    },
    /// A choice question's answer: one of its options.
    Choice {
        /// The id of the question answered.
        id: String,
        /// The `value` of the option picked.
        option: String,
    },
    /// A score question's answer: one of its levels.
    Score {
        /// The id of the question answered.
        id: String,
        /// The level picked: its number among the question's `levels`,
        /// counted from 0.
        level: u64,
    },
}

/// A noul question's answer.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Said {
    /// Yes.
    Yes,
    /// No.
    No,
}

/// The tool's answer: an empty object. The vote is in.
#[derive(Debug, Clone, Serialize)]
pub struct Voted {}

impl Vote {
    /// The id of the question answered, whichever kind it is.
    fn id(&self) -> &str {
        match self {
            Vote::Noul { id, .. } | Vote::Choice { id, .. } | Vote::Score { id, .. } => id,
        }
    }
}

/// Take the caller's vote on the decision `id`. The rules:
///
/// 1. **The decision is the caller's, and open.** `id` names an entry
///    of the caller's own queue. One that is another agent's, or is
///    gone, is no decision to the caller. An entry is gone once the
///    caller has voted on it or its decision has closed.
/// 2. **The vote answers the questions**, as [`answers`] holds it to.
///    A vote that does not is refused, and the entry is left where it
///    is, so the caller can vote again.
/// 3. **The vote takes the entry out.** Only then is it handed to the
///    call that waits for it. An entry already taken out, and a vote
///    nobody waits for, are a decision that has closed.
pub async fn call(state: &State, caller: &key::Agent, parameters: Parameters) -> Result<Voted, Failure> {
    let closed = || Failure::NoDecision(parameters.id.clone());
    // Rule 1.
    if parameters.id.len() != 8 || !parameters.id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(closed());
    }
    let entry = u32::from_str_radix(&parameters.id, 16).map_err(|_| closed())?;
    let pool = state.store.pool(state.daemon.client()).await?;
    let questions = queue::questions(pool, entry, &caller.template, caller.index).await?.ok_or_else(closed)?;
    let questions: Vec<Question> = serde_json::from_str(&questions).map_err(|error| Failure::Store(error.to_string()))?;
    // Rule 2.
    let answers = answers(&questions, &parameters.answers)?;
    // Rule 3.
    if !queue::claim_for(pool, entry, &caller.template, caller.index).await? {
        return Err(closed());
    }
    if !state.waiters.hand(entry, answers).await {
        return Err(closed());
    }
    Ok(Voted {})
}

/// The answers a vote makes, by question id, in the questions' order;
/// or why the vote is not one. The rules:
///
/// 1. **Every question is answered exactly once.** No two answers name
///    one question, no answer names a question the decision does not
///    ask, and no question is left without an answer.
/// 2. **An answer is of its question's kind.**
/// 3. **A noul's answer is certain**: `yes` is the noul `1`, and `no`
///    is the noul `0`.
/// 4. **A choice's answer is one option.** `option` is the value of
///    one of the question's options. The answer is a probability for
///    every option, in the options' order: `1` for the one picked and
///    `0` for each other.
/// 5. **A score's answer is one level.** `level` is less than the
///    number of the question's levels. The answer is a probability for
///    every level: `1` for the one picked and `0` for each other.
fn answers(questions: &[Question], votes: &[Vote]) -> Result<Answers, Failure> {
    // Rule 1.
    let mut given = HashMap::new();
    for vote in votes {
        if given.insert(vote.id(), vote).is_some() {
            return Err(Failure::Vote(format!("question {} is answered more than once", vote.id())));
        }
    }
    let mut answers = Answers::new();
    for question in questions {
        let id = question.id();
        let Some(vote) = given.remove(id) else {
            return Err(Failure::Vote(format!("question {id} is not answered, and every question must be")));
        };
        let answer = match (question, vote) {
            // Rule 3.
            (Question::Noul { .. }, Vote::Noul { answer, .. }) => Answer::Noul(structured_decisions::Noul {
                noul: match answer {
                    Said::Yes => Decimal::ONE,
                    Said::No => Decimal::ZERO,
                },
            }),
            // Rule 4.
            (Question::Choice { options, .. }, Vote::Choice { option, .. }) => {
                if !options.iter().any(|known| &known.value == option) {
                    return Err(Failure::Vote(format!("question {id} has no option {option}")));
                }
                Answer::Choice(structured_decisions::Choice {
                    probabilities: options.iter().map(|known| (known.value.clone(), if &known.value == option { Decimal::ONE } else { Decimal::ZERO })).collect(),
                })
            }
            // Rule 5.
            (Question::Score { levels, .. }, Vote::Score { level, .. }) => {
                if *level >= levels.len() as u64 {
                    return Err(Failure::Vote(format!("question {id} has {} levels, numbered from 0, and no level {level}", levels.len())));
                }
                Answer::Score(structured_decisions::Score {
                    probabilities: (0..levels.len() as u64).map(|known| if known == *level { Decimal::ONE } else { Decimal::ZERO }).collect(),
                })
            }
            // Rule 2.
            (Question::Noul { .. }, _) => return Err(Failure::Vote(format!("question {id} is a noul, and its answer must be of type noul"))),
            (Question::Choice { .. }, _) => return Err(Failure::Vote(format!("question {id} is a choice, and its answer must be of type choice"))),
            (Question::Score { .. }, _) => return Err(Failure::Vote(format!("question {id} is a score, and its answer must be of type score"))),
        };
        answers.insert(id.to_owned(), answer);
    }
    if let Some(stray) = given.keys().next() {
        return Err(Failure::Vote(format!("the decision asks no question {stray}")));
    }
    Ok(answers)
}

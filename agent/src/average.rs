//! The weighted average of the agents' answers.

use diverge_sdk::shared::containers::structured_decisions::{Answer, Choice, Noul, Question, Score};
use indexmap::IndexMap;
use rust_decimal::Decimal;

use crate::failure::Failure;
use crate::outcome::Outcome;

/// The answer to every question: the average of the agents' answers,
/// each weighed by its agent's weight.
///
/// `outcomes` and `weights` are in the same order, one per agent. The
/// rules:
///
/// 1. **An agent counts or it does not, whole.** An agent counts when
///    it answered and its answer fits every question, by rule 2. A
///    failure does not count, and neither does an answer that does
///    not fit: an agent is never counted for some questions and left
///    out of others, so every question is averaged over the same
///    agents.
/// 2. **An answer fits a question** when it is there under the
///    question's id and is of the question's kind: for a noul, a
///    `noul`; for a choice, a probability for every option and for
///    nothing else; for a score, a `score` and one probability per
///    level. Answers under ids no question has are passed over.
/// 3. **The weights are those of the agents that count**, divided by
///    their sum: an agent left out leaves its share to the rest, in
///    proportion.
/// 4. **Every number of an answer is averaged by itself**: the `noul`;
///    each option's probability; the `score`, and each level's
///    probability. Probabilities that sum to one in every answer sum
///    to one in the average.
/// 5. **With no agent that counts there is no answer**, and with a
///    number too large for a decimal there is none either.
///
/// The answers come back in the questions' order, a choice's options
/// in the question's order.
pub fn average(questions: &[Question], outcomes: &[Outcome], weights: &[Decimal]) -> Result<IndexMap<String, Answer>, Failure> {
    // Rules 1 and 2: the agents that count, each with its weight.
    let counted: Vec<(&IndexMap<String, Answer>, Decimal)> = outcomes
        .iter()
        .zip(weights)
        .filter_map(|(outcome, weight)| match outcome {
            Outcome::Answer(answers) if questions.iter().all(|question| fits(question, answers)) => Some((answers, *weight)),
            _ => None,
        })
        .collect();
    // Rule 5.
    if counted.is_empty() {
        return Err(Failure::NoAnswers);
    }
    // Rule 3.
    let total = counted.iter().try_fold(Decimal::ZERO, |total, (_, weight)| total.checked_add(*weight)).ok_or(Failure::Overflow)?;
    let shares: Vec<Decimal> = counted.iter().map(|(_, weight)| weight.checked_div(total)).collect::<Option<_>>().ok_or(Failure::Overflow)?;
    // Rule 4: one number of the answers, averaged. `of` reads it out
    // of an agent's answers; rule 2 has held that it is there.
    let mean = |of: &dyn Fn(&IndexMap<String, Answer>) -> Option<Decimal>| -> Result<Decimal, Failure> {
        counted.iter().zip(&shares).try_fold(Decimal::ZERO, |sum, ((answers, _), share)| {
            let value = of(answers).ok_or(Failure::NoAnswers)?;
            share.checked_mul(value).and_then(|part| sum.checked_add(part)).ok_or(Failure::Overflow)
        })
    };
    questions
        .iter()
        .map(|question| {
            let answer = match question {
                Question::Noul { id, .. } => Answer::Noul(Noul {
                    noul: mean(&|answers| match answers.get(id)? {
                        Answer::Noul(noul) => Some(noul.noul),
                        _ => None,
                    })?,
                }),
                Question::Choice { id, options, .. } => Answer::Choice(Choice {
                    probabilities: options
                        .iter()
                        .map(|option| {
                            let probability = mean(&|answers| match answers.get(id)? {
                                Answer::Choice(choice) => choice.probabilities.get(&option.value).copied(),
                                _ => None,
                            })?;
                            Ok((option.value.clone(), probability))
                        })
                        .collect::<Result<_, Failure>>()?,
                }),
                Question::Score { id, levels, .. } => Answer::Score(Score {
                    score: mean(&|answers| match answers.get(id)? {
                        Answer::Score(score) => Some(score.score),
                        _ => None,
                    })?,
                    probabilities: (0..levels.len())
                        .map(|level| {
                            mean(&|answers| match answers.get(id)? {
                                Answer::Score(score) => score.probabilities.get(level).copied(),
                                _ => None,
                            })
                        })
                        .collect::<Result<_, Failure>>()?,
                }),
            };
            Ok((id(question).to_owned(), answer))
        })
        .collect()
}

/// A question's id.
fn id(question: &Question) -> &str {
    match question {
        Question::Noul { id, .. } | Question::Choice { id, .. } | Question::Score { id, .. } => id,
    }
}

/// Whether an agent's answers hold an answer that fits the question:
/// rule 2 of [`average`].
fn fits(question: &Question, answers: &IndexMap<String, Answer>) -> bool {
    match (question, answers.get(id(question))) {
        (Question::Noul { .. }, Some(Answer::Noul(_))) => true,
        (Question::Choice { options, .. }, Some(Answer::Choice(choice))) => {
            choice.probabilities.len() == options.len() && options.iter().all(|option| choice.probabilities.contains_key(&option.value))
        }
        (Question::Score { levels, .. }, Some(Answer::Score(score))) => score.probabilities.len() == levels.len(),
        _ => false,
    }
}

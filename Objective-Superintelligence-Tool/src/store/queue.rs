//! The queue: a decision put in, read, and taken out.
//!
//! An entry is one decision in one voter's queue. The rules of its
//! life:
//!
//! 1. **It is made with its decision**, in the transaction that stores
//!    the decision: see [`decision`] and [`entry`].
//! 2. **It is read once.** [`next`] hands a voter its oldest unread
//!    entry and marks it read, so the next call hands the one after
//!    it, or nothing. A read entry stays, for the vote to find.
//! 3. **It is taken out once**, by whichever comes first: the voter's
//!    vote ([`claim_for`]) or the end of the voter's turn at the
//!    decision ([`claim`]). Taking it out is a delete, and only one
//!    delete finds the row, so a vote and an ending that cross are
//!    settled by the database.
//! 4. **Its decision is deleted when the call ends** ([`finish`]),
//!    and any entry still there goes with it.

use sqlx::{PgPool, Postgres, Row as _, Transaction};

/// Store a decision, its content and its questions each as JSON text;
/// its id.
pub async fn decision(transaction: &mut Transaction<'_, Postgres>, content: &str, questions: &str) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("insert into osi_decisions (content, questions) values ($1, $2) returning id").bind(content).bind(questions).fetch_one(&mut **transaction).await?;
    row.try_get("id")
}

/// Put `decision` in the queue of the agent `template` and `index`;
/// the entry's id, a random 32-bit number. An id already in use is
/// passed over and another is drawn.
pub async fn entry(transaction: &mut Transaction<'_, Postgres>, decision: i64, template: &str, index: u64) -> Result<u32, sqlx::Error> {
    loop {
        let row = sqlx::query(
            "insert into osi_entries (id, decision, agent_template, agent_index) values ((random() * 4294967295)::bigint, $1, $2, $3)
             on conflict (id) do nothing returning id",
        )
        .bind(decision)
        .bind(template)
        .bind(index as i64)
        .fetch_optional(&mut **transaction)
        .await?;
        if let Some(row) = row {
            return Ok(row.try_get::<i64, _>("id")? as u32);
        }
    }
}

/// An entry as a voter reads it: its id, and its decision's content
/// and questions as the JSON text they were stored as.
pub struct Read {
    /// The entry's id.
    pub id: u32,
    /// The decision's content.
    pub content: String,
    /// The decision's questions.
    pub questions: String,
}

/// The oldest unread entry of the agent `template` and `index`, marked
/// read; or none.
pub async fn next(pool: &PgPool, template: &str, index: u64) -> Result<Option<Read>, sqlx::Error> {
    let row = sqlx::query(
        "with taken as (
            update osi_entries set read = true where id = (
                select id from osi_entries where agent_template = $1 and agent_index = $2 and not read
                order by position limit 1 for update skip locked
            ) returning id, decision
        )
        select taken.id, osi_decisions.content, osi_decisions.questions from taken join osi_decisions on osi_decisions.id = taken.decision",
    )
    .bind(template)
    .bind(index as i64)
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else { return Ok(None) };
    Ok(Some(Read {
        id: row.try_get::<i64, _>("id")? as u32,
        content: row.try_get("content")?,
        questions: row.try_get("questions")?,
    }))
}

/// The questions of the entry `id`, as JSON text, where the entry is
/// the agent `template` and `index`'s; or none. Another agent's entry
/// is none, as an absent one is.
pub async fn questions(pool: &PgPool, id: u32, template: &str, index: u64) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query(
        "select osi_decisions.questions from osi_entries join osi_decisions on osi_decisions.id = osi_entries.decision
         where osi_entries.id = $1 and osi_entries.agent_template = $2 and osi_entries.agent_index = $3",
    )
    .bind(i64::from(id))
    .bind(template)
    .bind(index as i64)
    .fetch_optional(pool)
    .await?;
    row.map(|row| row.try_get("questions")).transpose()
}

/// Take the entry `id` out, for its vote, where it is the agent
/// `template` and `index`'s; whether it was there to take.
pub async fn claim_for(pool: &PgPool, id: u32, template: &str, index: u64) -> Result<bool, sqlx::Error> {
    let deleted = sqlx::query("delete from osi_entries where id = $1 and agent_template = $2 and agent_index = $3").bind(i64::from(id)).bind(template).bind(index as i64).execute(pool).await?;
    Ok(deleted.rows_affected() == 1)
}

/// Take the entry `id` out, at the end of its voter's turn; whether it
/// was there to take.
pub async fn claim(pool: &PgPool, id: u32) -> Result<bool, sqlx::Error> {
    let deleted = sqlx::query("delete from osi_entries where id = $1").bind(i64::from(id)).execute(pool).await?;
    Ok(deleted.rows_affected() == 1)
}

/// Delete the decision, and with it any entry of it still there.
pub async fn finish(pool: &PgPool, decision: i64) -> Result<(), sqlx::Error> {
    sqlx::query("delete from osi_decisions where id = $1").bind(decision).execute(pool).await?;
    Ok(())
}

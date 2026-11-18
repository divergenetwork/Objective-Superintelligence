//! The tables, made where absent, and emptied.

use sqlx::PgPool;

/// Every statement that readies the queue, in order. The rules:
///
/// 1. **Each table is made where absent**, so ensuring it again is
///    nothing.
/// 2. **A decision is stored once.** `osi_decisions` holds its content
///    and its questions, each as the JSON text the call carried, so
///    the order of every member is kept. `osi_entries` holds one row
///    per voter the decision is put to, and refers to it.
/// 3. **An entry's id is a random 32-bit number**, held in a `bigint`
///    since Postgres has no unsigned type. `position` is the order
///    entries were made in, which is the order a queue is read in.
/// 4. **Whatever is there is deleted.** A decision lives only as long
///    as the call that put it, and that call is this program's. Rows
///    found when the pool is first made are a previous run's, whose
///    calls are gone.
const STATEMENTS: [&str; 4] = [
    "create table if not exists osi_decisions (
        id bigserial primary key,
        content text not null,
        questions text not null
    )",
    "create table if not exists osi_entries (
        id bigint primary key,
        position bigserial not null,
        decision bigint not null references osi_decisions (id) on delete cascade,
        agent_template text not null,
        agent_index bigint not null,
        read boolean not null default false
    )",
    "create index if not exists osi_entries_by_agent on osi_entries (agent_template, agent_index, read, position)",
    "delete from osi_decisions",
];

/// Ready the queue: see [`STATEMENTS`].
pub async fn ensure(pool: &PgPool) -> Result<(), sqlx::Error> {
    for statement in STATEMENTS {
        sqlx::query(statement).execute(pool).await?;
    }
    Ok(())
}

//! The voters' queues, in the tool's database.
//!
//! Two tables: [`schema`] makes them; [`queue`] puts a decision in
//! every voter's queue, hands a voter the next one, and takes entries
//! out again. A decision is stored once, however many queues it is in.

pub mod queue;
mod schema;

use diverge_sdk::container_proxy::inside::Client;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::OnceCell;

use crate::failure::Failure;

/// The pool, made on the first use that needs the database, with the
/// tables ensured and emptied. A failure leaves nothing behind, so the
/// next use tries again.
#[derive(Default)]
pub struct Store {
    pool: OnceCell<PgPool>,
}

impl Store {
    /// The pool, through the proxy's loopback pgwire.
    pub async fn pool(&self, client: &Client) -> Result<&PgPool, Failure> {
        self.pool
            .get_or_try_init(|| async {
                let pool = PgPoolOptions::new().max_connections(8).connect(&client.postgres_url()).await?;
                schema::ensure(&pool).await?;
                Ok::<_, Failure>(pool)
            })
            .await
    }
}

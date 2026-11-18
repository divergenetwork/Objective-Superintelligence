//! The daemon, as the tool reaches it.

use diverge_sdk::container_proxy::inside::Client;
use diverge_sdk::wire::client::handle::Handle;
use tokio::sync::Mutex;

use crate::failure::Failure;

/// The tool's one connection to the daemon, through the proxy beside
/// it: dialled when a call first needs it, kept, and dialled again
/// after it is found dead.
#[derive(Default)]
pub struct Daemon {
    /// The proxy, from the inside.
    client: Client,
    /// The connection, once made.
    handle: Mutex<Option<Handle>>,
}

impl Daemon {
    /// The proxy, from the inside: what the database is reached
    /// through as well.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// The connection: the one held, or a new one.
    pub async fn handle(&self) -> Result<Handle, Failure> {
        let mut held = self.handle.lock().await;
        if let Some(handle) = held.as_ref() {
            return Ok(handle.clone());
        }
        let handle = self.client.daemon().await.map_err(|error| Failure::Daemon(error.to_string()))?;
        *held = Some(handle.clone());
        Ok(handle)
    }

    /// Forget the connection, so the next call dials: what a request
    /// that could not even be opened on it calls for.
    pub async fn forget(&self) {
        *self.handle.lock().await = None;
    }
}

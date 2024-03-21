use crate::ChatError;
use std::future::Future;
use tokio::task::JoinHandle;
use tracing::error;

/// Spawns a new task, logs errors if encountered.
pub fn spawn<F>(future: F) -> JoinHandle<()>
where
    F: Future<Output = Result<(), ChatError>> + Send + 'static,
{
    tokio::spawn(async move {
        if let Err(err) = future.await {
            error!(cause = %err, "task failed");
        }
    })
}

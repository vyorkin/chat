use thiserror::Error;
use tracing::subscriber::SetGlobalDefaultError;

#[derive(Debug, Error)]
pub enum ChatError {
    #[error(transparent)]
    TelemetryError(#[from] SetGlobalDefaultError),

    #[error("Peer disconnected immediately")]
    PeerDisconnectedError,

    #[error("Message could not be empty")]
    EmptyMessageError(String),
}

use thiserror::Error;
use tracing_subscriber::util::TryInitError;

#[derive(Debug, Error)]
pub enum ChatError {
    #[error(transparent)]
    TelemetryError(#[from] TryInitError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("Peer disconnected immediately")]
    PeerDisconnectedError,
}

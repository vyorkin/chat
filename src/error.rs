use crate::event::Event;
use thiserror::Error;
use tokio::sync::mpsc;
use tracing_subscriber::util::TryInitError;

#[derive(Debug, Error)]
pub enum ChatError {
    #[error(transparent)]
    TelemetryError(#[from] TryInitError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    SendEventError(#[from] mpsc::error::SendError<Event>),

    #[error(transparent)]
    SendMessageError(#[from] mpsc::error::SendError<String>),

    #[error(transparent)]
    SendShutdownError(#[from] mpsc::error::SendError<()>),

    #[error("Peer disconnected immediately")]
    PeerDisconnectedError,
}

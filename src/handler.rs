use crate::{connection::Connection, ChatError, Event, EventSender, Shutdown};
use tokio::net::tcp::OwnedWriteHalf;
use tracing::instrument;

/// Per-connection handler.
///
/// Processes requests from the connection until the peer disconnects or
/// a shutdown notification is received from `shutdown`.
pub struct Handler {
    /// The TCP connection wrapper.
    connection: Connection,
    /// A shutdown notification.
    shutdown: Shutdown,
}

impl Handler {
    pub fn new(connection: Connection, shutdown: Shutdown) -> Self {
        Self {
            connection,
            shutdown,
        }
    }

    #[instrument(skip(self, socket_writer))]
    pub async fn run(
        &mut self,
        socket_writer: OwnedWriteHalf,
        event_sender: EventSender,
    ) -> Result<(), ChatError> {
        let name = self.connection.read_name().await?;

        event_sender.send(Event::NewPeer {
            name: name.clone(),
            socket_writer,
        })?;

        while let Some(event) = self.connection.read_event(name.clone()).await? {
            event_sender.send(event)?;
        }

        Ok(())
    }
}

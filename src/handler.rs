use crate::{connection::Connection, ChatError, Event, EventSender, Shutdown};
use tokio::{net::tcp::OwnedWriteHalf, select};
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
        // Read peer's name first.
        let name = self.connection.read_name().await?;

        let _ = event_sender.send(Event::NewPeer {
            name: name.clone(),
            socket_writer,
        });

        // As long as the shutdown signal has not been received, try to read next event.
        while !self.shutdown.is_shutdown() {
            let maybe_event = select! {
                res = self.connection.read_event(name.clone()) => res?,
                _ = self.shutdown.recv() => {
                    // If a shutdown signal is received, return from `run`.
                    // This will result in the task termination.
                    return Ok(());
                }
            };

            let event = match maybe_event {
                Some(event) => event,
                None => return Ok(()),
            };

            let _ = event_sender.send(event);
        }

        Ok(())
    }
}

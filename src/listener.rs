use crate::{
    connection::Connection, handler::Handler, shutdown::Shutdown, utils, ChatError, EventSender,
};
use tokio::{net::TcpListener, sync::broadcast};
use tracing::instrument;

pub struct Listener {
    /// TCP listener supplied by the `run` caller.
    tcp_listener: TcpListener,

    /// Broadcasts a shutdown signal to all active connections.
    ///
    /// When a connection task is spawned, it is passed a broadcast receiver handle.
    /// When a graceful shutdown is initiated, a `()` value is sent via this `notify_shutdown` broadcast sender.
    /// Each active connection receives it, reaches a safe terminal state, and completes the task.
    notify_shutdown: broadcast::Sender<()>,
}

impl Listener {
    pub fn new(tcp_listener: TcpListener, notify_shutdown: broadcast::Sender<()>) -> Self {
        Self {
            tcp_listener,
            notify_shutdown,
        }
    }

    #[instrument(skip(self))]
    pub async fn run(&mut self, event_sender: EventSender) -> Result<(), ChatError> {
        loop {
            let event_sender = event_sender.clone();

            let (tcp_stream, _) = self.tcp_listener.accept().await?;
            let (socket_reader, socket_writer) = tcp_stream.into_split();

            let connection = Connection::new(socket_reader);
            let shutdown = Shutdown::new(self.notify_shutdown.subscribe());

            // Create the per-connection handler.
            let mut handler = Handler::new(connection, shutdown);
            // Spawn a new task to handle connection.
            utils::spawn(async move { handler.run(socket_writer, event_sender).await });
        }
    }
}

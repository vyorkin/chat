use chat::{telemetry, ChatError};
use color_eyre::eyre::Result;
use std::{collections::HashMap, future::Future, net::SocketAddr};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{tcp::OwnedWriteHalf, TcpListener, TcpStream, ToSocketAddrs},
    select,
    sync::mpsc,
    task::JoinHandle,
};
use tracing::{error, info, instrument};

enum Event {
    NewPeer {
        name: String,
        stream: OwnedWriteHalf,
    },
    BroadcastMessage {
        from: String,
        text: String,
    },
    PeerMessage {
        from: String,
        to: Vec<String>,
        text: String,
    },
    Shutdown,
}

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

type EventSender = Sender<Event>;
type EventReceiver = Receiver<Event>;

type MessageSender = Sender<String>;
type MessageReceiver = Receiver<String>;

#[instrument(name = "Processing connection", skip(socket, events))]
pub async fn process_connection(
    socket: TcpStream,
    address: SocketAddr,
    events: EventSender,
) -> Result<()> {
    let (socket_reader, socket_writer) = socket.into_split();

    let reader = BufReader::new(socket_reader);
    let mut lines = reader.lines();

    let name = match lines.next_line().await? {
        Some(line) => Ok(line),
        None => Err(ChatError::PeerDisconnectedError),
    }?;

    info!("<- {name} joined");
    events.send(Event::NewPeer {
        name: name.clone(),
        stream: socket_writer,
    })?;

    while let Some(line) = lines.next_line().await? {
        info!("line: {}", line);

        if line.trim() == "shutdown" {
            info!("Shutdown requested");
            events.send(Event::Shutdown)?;
            continue;
        }

        let (receivers, text) = match line.split_once(':') {
            Some((receivers, text)) => (Some(receivers), text.trim()),
            None => (None, line.trim()),
        };
        let text = text.to_owned();
        match receivers {
            Some(receivers) => {
                let receivers: Vec<String> =
                    receivers.split(',').map(|s| s.trim().to_owned()).collect();

                info!(
                    "PeerMessage: {} -> {}: {}",
                    name,
                    receivers.join(", "),
                    text
                );
                events.send(Event::PeerMessage {
                    from: name.clone(),
                    to: receivers,
                    text,
                })?;
            }
            None => {
                info!("BroadcastMessage: {}: {}", name, text);
                events.send(Event::BroadcastMessage {
                    from: name.clone(),
                    text,
                })?;
            }
        }
    }

    Ok(())
}

#[instrument(name = "Processing events", skip(events, shutdown))]
pub async fn process_events(mut events: EventReceiver, shutdown: Sender<()>) -> Result<()> {
    use std::collections::hash_map::Entry;

    let mut peers: HashMap<String, MessageSender> = HashMap::new();

    while let Some(event) = events.recv().await {
        match event {
            Event::NewPeer { name, stream } => match peers.entry(name) {
                Entry::Occupied(_) => (),
                Entry::Vacant(entry) => {
                    let (message_tx, message_rx) = mpsc::unbounded_channel();
                    entry.insert(message_tx);
                    spawn(process_messages(stream, message_rx));
                }
            },
            Event::PeerMessage { from, to, text } => {
                for receiver in to {
                    if let Some(peer) = peers.get_mut(&receiver) {
                        let message = format!("{}: {}\n", from, text);
                        peer.send(message)?;
                    }
                }
            }
            Event::BroadcastMessage { from, text } => {
                for peer in peers.values() {
                    let message = format!("{}: {}\n", from, text);
                    peer.send(message)?;
                }
            }
            Event::Shutdown => {
                info!("Sending shutdown message to channel");
                shutdown.send(())?;
            }
        }
    }

    Ok(())
}

#[instrument(name = "Processing messages")]
pub async fn process_messages(
    mut socket_writer: OwnedWriteHalf,
    mut messages: MessageReceiver,
) -> Result<()> {
    while let Some(message) = messages.recv().await {
        socket_writer.write_all(message.as_bytes()).await?;
    }

    Ok(())
}

#[instrument(name = "Listening", skip(address))]
async fn listen<A: ToSocketAddrs>(address: A) -> std::io::Result<()> {
    let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel::<()>();
    let (event_tx, event_rx) = mpsc::unbounded_channel::<Event>();

    let listener = TcpListener::bind(address).await?;
    spawn(process_events(event_rx, shutdown_tx));

    loop {
        select! {
            Ok((socket, socket_addr)) = listener.accept() => {
                spawn(process_connection(socket, socket_addr, event_tx.clone()));
            },
            _ = tokio::signal::ctrl_c() => {
                info!("Got CTRL+C");
                break;
            },
            Some(()) = shutdown_rx.recv() => {
                info!("Shutdown acknowledged");
                break;
            }
        }
    }

    Ok(())
}

fn spawn<F>(future: F) -> JoinHandle<()>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    tokio::spawn(async move {
        if let Err(e) = future.await {
            error!("{}", e)
        }
    })
}

#[tokio::main]
pub async fn main() -> Result<()> {
    color_eyre::install()?;
    telemetry::setup("info")?;
    listen("127.0.0.1:6969").await?;
    Ok(())
}

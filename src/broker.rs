use crate::{event::Event, utils, ChatError, EventReceiver, MessageSender, Sender};
use std::collections::HashMap;
use tokio::{io::AsyncWriteExt, sync::mpsc};
use tracing::{info, instrument};

pub struct Broker {
    peers: HashMap<String, MessageSender>,
    event_receiver: EventReceiver,
    shutdown: Sender<()>,
}

impl Broker {
    pub fn new(event_receiver: EventReceiver, shutdown: Sender<()>) -> Self {
        let peers: HashMap<String, MessageSender> = HashMap::new();
        Self {
            peers,
            event_receiver,
            shutdown,
        }
    }

    #[instrument(skip(self))]
    pub async fn run(&mut self) -> Result<(), ChatError> {
        use std::collections::hash_map::Entry;

        while let Some(event) = self.event_receiver.recv().await {
            match event {
                Event::NewPeer {
                    name,
                    mut socket_writer,
                } => match self.peers.entry(name.clone()) {
                    Entry::Occupied(_) => (),
                    Entry::Vacant(entry) => {
                        info!("{name} joined");
                        let (message_tx, mut message_rx) = mpsc::unbounded_channel();
                        entry.insert(message_tx);
                        utils::spawn(async move {
                            while let Some(message) = message_rx.recv().await {
                                socket_writer.write_all(message.as_bytes()).await?;
                            }
                            Ok(())
                        });
                    }
                },
                Event::DirectMessage { message, to } => {
                    info!(
                        "[direct]: {} -> {}: {}",
                        message.from,
                        to.join(", "),
                        message.text
                    );
                    for receiver in to {
                        if let Some(peer) = self.peers.get_mut(&receiver) {
                            peer.send(format!("{}\n", message))?;
                        }
                    }
                }
                Event::BroadcastMessage(message) => {
                    info!("[broadcast] {}", message.to_string());
                    for peer in self.peers.values() {
                        peer.send(format!("{}\n", message))?;
                    }
                }
                Event::Shutdown => {
                    info!("shutdown requested");
                    self.shutdown.send(())?;
                }
            }
        }

        Ok(())
    }
}

use crate::message::Message;
use tokio::net::tcp::OwnedWriteHalf;

#[derive(Debug)]
pub enum Event {
    NewPeer {
        name: String,
        socket_writer: OwnedWriteHalf,
    },
    DirectMessage {
        message: Message,
        to: Vec<String>,
    },
    BroadcastMessage(Message),
    Shutdown,
}

impl Event {
    pub fn from_line(name: String, line: String) -> Self {
        if line == "shutdown" {
            return Event::Shutdown;
        }

        let (receivers, text) = match line.split_once(':') {
            Some((receivers, text)) => (Some(receivers), text.trim().to_owned()),
            None => (None, line),
        };
        let text = text.to_owned();
        match receivers {
            Some(receivers) => {
                let receivers: Vec<String> =
                    receivers.split(',').map(|s| s.trim().to_owned()).collect();

                Event::DirectMessage {
                    message: Message::new(name.clone(), text),
                    to: receivers,
                }
            }
            None => Event::BroadcastMessage(Message::new(name.clone(), text)),
        }
    }
}

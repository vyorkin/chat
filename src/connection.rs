use crate::{ChatError, Event};
use tokio::{
    io::{AsyncBufReadExt, BufReader, Lines},
    net::tcp::OwnedReadHalf,
};

/// Reads `Event`'s  from a remote peer with buffering.
#[derive(Debug)]
pub struct Connection {
    lines: Lines<BufReader<OwnedReadHalf>>,
}

impl Connection {
    pub fn new(socket_reader: OwnedReadHalf) -> Self {
        let reader = BufReader::new(socket_reader);
        let lines = reader.lines();
        Self { lines }
    }

    pub async fn read_name(&mut self) -> Result<String, ChatError> {
        match self.lines.next_line().await? {
            Some(line) => Ok(line),
            None => Err(ChatError::PeerDisconnectedError),
        }
    }

    pub async fn read_event(&mut self, name: String) -> Result<Option<Event>, ChatError> {
        let line = self.lines.next_line().await?;
        match line {
            Some(s) => Ok(Some(Event::from_line(name, s))),
            None => Ok(None),
        }
    }
}

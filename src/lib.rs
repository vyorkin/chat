use tokio::sync::mpsc;

mod error;
mod event;

mod broker;
mod connection;
mod handler;
mod listener;
mod message;
mod shutdown;

pub mod telemetry;
pub mod utils;

pub use broker::Broker;
pub use error::ChatError;
pub use event::Event;
pub use handler::Handler;
pub use listener::Listener;
pub use message::Message;
pub use shutdown::Shutdown;

pub type Sender<T> = mpsc::UnboundedSender<T>;
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;

pub type EventSender = Sender<Event>;
pub type EventReceiver = Receiver<Event>;

pub type MessageSender = Sender<String>;
pub type MessageReceiver = Receiver<String>;

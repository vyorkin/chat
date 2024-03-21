use tracing::Level;
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt, Registry};

use crate::ChatError;

pub fn setup(_env: &str) -> Result<(), ChatError> {
    // let console_layer = console_subscriber::spawn();

    let filter = filter::Targets::new().with_target("server", Level::INFO);

    Registry::default()
        // .with(console_layer)
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .try_init()?;

    Ok(())
}

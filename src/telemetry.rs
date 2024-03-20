use tracing::Level;
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt, Registry};

use crate::ChatError;

pub fn setup(_env: &str) -> Result<(), ChatError> {
    // let env_filter =
    //     EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env));
    // let subscriber = Registry::default().with(env_filter).with(fmt::layer());

    // let subscriber = tracing_subscriber::FmtSubscriber::new();
    // tracing::subscriber::set_global_default(subscriber)?;

    // let subscriber = tracing_subscriber::fmt()
    //     .compact()
    //     .with_file(true)
    //     .with_line_number(true)
    //     .with_thread_ids(true)
    //     .with_target(false)
    //     .finish();
    // tracing::subscriber::set_global_default(subscriber)?;

    // tracing_subscriber::fmt::init();
    // console_subscriber::init();

    // let console_layer = console_subscriber::spawn();

    // let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let filter = filter::Targets::new().with_target("server", Level::INFO);

    Registry::default()
        // .with(console_layer)
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    Ok(())
}

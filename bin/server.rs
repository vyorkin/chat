use chat::{telemetry, Broker, Event, Listener};
use clap::{Parser, Subcommand};
use color_eyre::eyre;
use std::future::Future;
use tokio::{
    net::{TcpListener, ToSocketAddrs},
    select, signal,
    sync::{broadcast, mpsc},
};
use tracing::{error, info, instrument};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Start {
        #[arg(short, long)]
        port: Option<u16>,
    },
}

#[instrument(skip(address, shutdown))]
async fn run<A: ToSocketAddrs, S: Future>(address: A, shutdown: S) -> std::io::Result<()> {
    let (notify_shutdown, _) = broadcast::channel::<()>(1);

    let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel::<()>();
    let (event_sender, event_receiver) = mpsc::unbounded_channel::<Event>();

    let tcp_listener = TcpListener::bind(address).await?;
    let mut listener = Listener::new(tcp_listener, notify_shutdown);
    let mut broker = Broker::new(event_receiver, shutdown_tx);

    // Concurrently wait on listener and shutdown signal.
    // The listener can only complete if an error is encountered.
    // So under normal circumstances, this `select!` statement
    // completes when shutdown signal is received.
    select! {
        res = listener.run(event_sender) => {
            if let Err(err) = res {
                error!(cause = %err, "listener failure");
            }
        },
        res = broker.run() => {
            if let Err(err) = res {
                error!(cause = %err, "broker failure");
            }
        },
        _ = shutdown => {
            info!("shutting down");
        }
    }

    Ok(())
}

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    telemetry::setup("info")?;

    let cli = Cli::parse();
    match cli.command {
        Commands::Start { port } => {
            let port = port.unwrap_or(6969);
            let address = format!("127.0.0.1:{}", port);

            run(address, signal::ctrl_c()).await?;
        }
    }

    Ok(())
}

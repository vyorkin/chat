use chat::telemetry;
use color_eyre::eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    telemetry::setup("info")?;

    Ok(())
}

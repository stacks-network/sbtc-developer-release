use clap::Parser;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = romeo::config::Cli::parse();
    let config = romeo::config::Config::from_args(args)?;
    let state = romeo::state::State::default();

    romeo::system::run(config, state).await;

    Ok(())
}

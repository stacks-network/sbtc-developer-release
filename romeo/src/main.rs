use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::registry()
		.with(tracing_subscriber::fmt::layer())
		.with(tracing_subscriber::EnvFilter::from_default_env())
		.init();

	let args = romeo::config::Cli::parse();
	let config = romeo::config::Config::from_path(args.config_file)?;

	romeo::system::run(config).await;

	Ok(())
}

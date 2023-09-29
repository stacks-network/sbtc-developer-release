use clap::Parser;
use tracing_subscriber::{
	filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::registry()
		.with(tracing_subscriber::fmt::layer().compact().with_ansi(false))
		.with(
			tracing_subscriber::EnvFilter::builder()
				.with_default_directive(LevelFilter::INFO.into())
				.from_env_lossy(),
		)
		.init();

	let args = romeo::config::Cli::parse();
	let config = romeo::config::Config::from_path(args.config_file)?;

	romeo::system::run(config).await;

	Ok(())
}

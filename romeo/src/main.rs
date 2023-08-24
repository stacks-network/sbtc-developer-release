use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = romeo::config::Cli::parse();
    let config = romeo::config::Config::from_args(args)?;
    let state = romeo::state::State::default();

    romeo::system::run(config, state).await;

    Ok(())
}

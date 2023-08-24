use std::time::Duration;

use clap::Parser;

use romeo::store::Store;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = romeo::config::Cli::parse();
    let config = romeo::config::Config::from_args(args)?;

    todo!();

    Ok(())
}

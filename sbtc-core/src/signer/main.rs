use clap::Parser;
use sbtc_core::signer::{coordinator::fire::Coordinator as FireCoordinator, FrostSigner, Signer};
use tracing::error;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Config file path
    #[arg(short, long)]
    config: String,
}

fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match Signer::<FrostSigner, FireCoordinator>::from_path(&cli.config) {
        Ok(signer) => {
            if let Err(e) = signer.run() {
                error!("An error occurred running the signer: {}", e);
            }
        }
        Err(e) => {
            error!(
                "An error occurred loading config file {}: {}",
                cli.config, e
            );
        }
    }
}

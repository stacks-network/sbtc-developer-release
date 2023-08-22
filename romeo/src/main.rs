use std::time::Duration;

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = romeo::config::Cli::parse();

    let config: romeo::config::Config = {
        let file = std::fs::File::open(&args.config_file)?;
        serde_json::from_reader(file)?
    };

    let state_directory = args
        .config_file
        .parent()
        .unwrap()
        .join(&config.state_directory);

    let mut system = romeo::actor::System::new(state_directory);

    system.spawn::<romeo::deposit::DepositProcessor>();
    system.spawn::<romeo::contract_deployer::ContractDeployer>();

    system
        .tick_and_wait(Duration::from_secs(config.tick_interval_seconds))
        .await;

    Ok(())
}

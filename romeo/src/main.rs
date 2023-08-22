use std::time::Duration;

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = romeo::config::Cli::parse();
    let config = romeo::config::Config::from_args(args)?;
    let mut system = romeo::actor::System::new(config.state_directory);

    system.spawn::<romeo::deposit::DepositProcessor>();
    system.spawn::<romeo::contract_deployer::ContractDeployer>();

    system
        .tick_and_wait(Duration::from_secs(config.tick_interval_seconds))
        .await;

    Ok(())
}

use std::time::Duration;

use clap::Parser;

use romeo::store::Store;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = romeo::config::Cli::parse();
    let config = romeo::config::Config::from_args(args)?;
    let store = romeo::store::FileStore::new(config.state_directory);
    let mut system = romeo::actor::System::new(store.clone());

    let deposit_processor: romeo::deposit::DepositProcessor = store.read().expect("Failed to read deposit processor").unwrap_or_default();

    let contract_deployer: romeo::contract_deployer::ContractDeployer= store.read().expect("Failed to read contract deployer").unwrap_or_default();


    system.spawn(deposit_processor);
    system.spawn(contract_deployer);

    system
        .tick_and_wait(Duration::from_secs(config.tick_interval_seconds))
        .await;

    Ok(())
}

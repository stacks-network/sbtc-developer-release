use bdk::bitcoin::Txid as BitcoinTxId;
use blockstack_lib::burnchains::Txid as StacksTxId;
use tokio::sync::mpsc;
use tracing::debug;

use crate::config::Config;
use crate::event::Event;
use crate::event::TransactionStatus;
use crate::state;
use crate::task::Task;

pub async fn run(config: Config, mut state: state::State) {
    let (tx, mut rx) = mpsc::channel::<Event>(128); // TODO: Make capacity configurable

    // Bootstrap
    spawn(config.clone(), Task::CreateAssetContract, tx.clone());

    while let Some(event) = rx.recv().await {
        let (next_state, tasks) = state::update(&config, state, event);

        for task in tasks {
            spawn(config.clone(), task, tx.clone());
        }

        state = next_state;
    }
}

#[tracing::instrument(skip(config, result))]
fn spawn(config: Config, task: Task, result: mpsc::Sender<Event>) -> tokio::task::JoinHandle<()> {
    debug!("Spawning task");

    tokio::task::spawn(async move {
        let event = run_task(&config, task).await;
        result.send(event).await.expect("Failed to return event");
    })
}

async fn run_task(config: &Config, task: Task) -> Event {
    match task {
        Task::CreateAssetContract => deploy_asset_contract(config).await,
        Task::CheckBitcoinTransactionStatus(txid) => {
            check_bitcoin_transaction_status(config, txid).await
        }
        Task::CheckStacksTransactionStatus(txid) => {
            check_stacks_transaction_status(config, txid).await
        }
        Task::FetchBitcoinBlock(block_height) => fetch_bitcoin_block(config, block_height).await,
        _ => panic!(),
    }
}

async fn deploy_asset_contract(_config: &Config) -> Event {
    // TODO: #73
    println!("Deploying");
    Event::AssetContractCreated(StacksTxId([0; 32]))
}

async fn check_bitcoin_transaction_status(_config: &Config, _txid: BitcoinTxId) -> Event {
    todo!();
}

async fn check_stacks_transaction_status(_config: &Config, txid: StacksTxId) -> Event {
    // TODO

    Event::StacksTransactionUpdate(txid, TransactionStatus::Rejected)
}

async fn fetch_bitcoin_block(_config: &Config, _block_height: u64) -> Event {
    todo!();
}

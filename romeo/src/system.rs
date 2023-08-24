use bdk::bitcoin::{Transaction as BitcoinTransaction, Txid as BitcoinTxId};
use blockstack_lib::{burnchains::Txid as StacksTxId, chainstate::stacks::StacksTransaction};
use tokio::sync::mpsc;

use crate::config::Config;
use crate::event::Event;
use crate::state;
use crate::task::Task;

pub async fn run(config: Config, mut state: state::State) {
    let (tx, mut rx) = mpsc::channel::<Event>(128); // TODO: Make capacity configurable

    while let Some(event) = rx.recv().await {
        let (next_state, tasks) = state::update(&config, state, event);

        for task in tasks {
            spawn(config.clone(), task, tx.clone());
        }

        state = next_state;
    }
}

fn spawn(config: Config, task: Task, result: mpsc::Sender<Event>) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn(async move {
        let event = run_task(&config, task).await;
        result.send(event).await.expect("Failed to return event");
    })
}

async fn run_task(config: &Config, task: Task) -> Event {
    match task {
        Task::BroadcastBitcoinTransaction(transaction) => {
            broadcast_bitcoin_transaction(config, transaction).await
        }
        Task::BroadcastStacksTransaction(transaction) => {
            broadcast_stacks_transaction(config, transaction).await
        }
        Task::CheckBitcoinTransactionStatus(txid) => {
            check_bitcoin_transaction_status(config, txid).await
        }
        Task::CheckStacksTransactionStatus(txid) => {
            check_stacks_transaction_status(config, txid).await
        }
        Task::FetchBitcoinBlock(block_height) => fetch_bitcoin_block(config, block_height).await,
    }
}

async fn broadcast_bitcoin_transaction(config: &Config, transaction: BitcoinTransaction) -> Event {
    todo!();
}

async fn broadcast_stacks_transaction(config: &Config, transaction: StacksTransaction) -> Event {
    todo!();
}

async fn check_bitcoin_transaction_status(config: &Config, txid: BitcoinTxId) -> Event {
    todo!();
}

async fn check_stacks_transaction_status(config: &Config, txid: StacksTxId) -> Event {
    todo!();
}

async fn fetch_bitcoin_block(config: &Config, block_height: u64) -> Event {
    todo!();
}

use bdk::bitcoin::{Block, Transaction as BitcoinTransaction, Txid as BitcoinTxId};
use blockstack_lib::burnchains::Txid as StacksTxId;
use blockstack_lib::chainstate::stacks::StacksTransaction;
use blockstack_lib::types::chainstate::StacksAddress;

use crate::config::Config;
use crate::task::Task;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum TransactionStatus {
    Created,
    Broadcasted,
    Confirmed,
    Rejected,
}

#[derive(Debug, Clone)]
pub enum Event {
    StacksTransactionUpdate(StacksTxId, TransactionStatus),
    BitcoinTransactionUpdate(StacksTxId, TransactionStatus),

    BitcoinBlock(Block),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct State {
    deposits: Vec<Deposit>,
    withdrawals: Vec<Withdrawal>,
    next_stx_nonce: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Response<T> {
    tx: T,
    status: TransactionStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Deposit {
    txid: BitcoinTxId,
    amount: u64,
    recipient: StacksAddress,
    block_height: u64,
    mint: Option<Response<StacksTransaction>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Withdrawal {
    txid: BitcoinTxId,
    amount: u64,
    recipient: StacksAddress,
    block_height: u64,
    burn: Option<Response<StacksTransaction>>,
    fulfillment: Option<Response<BitcoinTransaction>>,
}

pub fn update(config: Config, state: State, event: Event) -> (State, Vec<Task>) {
    match event {
        Event::StacksTransactionUpdate(_, _) => todo!(),
        Event::BitcoinTransactionUpdate(_, _) => todo!(),
        Event::BitcoinBlock(_) => todo!(),
    }
}

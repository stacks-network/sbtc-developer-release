use bdk::bitcoin::{Block, Txid as BitcoinTxId};
use blockstack_lib::burnchains::Txid as StacksTxId;
use blockstack_lib::chainstate::stacks::StacksTransaction;
use blockstack_lib::types::chainstate::StacksAddress;

use crate::task::Task;

#[derive(Debug, Clone)]
pub enum Event {
    StacksTransactionUpdate(StacksTxId),
    BitcoinTransactionUpdate(StacksTxId),

    BitcoinBlock(Block),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct State {
    deposits: Vec<Deposit>,
    next_stx_nonce: u64,
}

struct Deposit {
    txid: BitcoinTxId,
    amount: u64,
    recipient: StacksAddress,
    block_height: u64,
    responses: Vec<ResponseTx>,
}

struct ResponseTx {
    txid: StacksTxId,
    tx: StacksTransaction,
    status: TransactionStatus,
}

pub fn update(state: State, event: Event) -> (State, Vec<Task>) {
    todo!();
}

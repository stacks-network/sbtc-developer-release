use bdk::bitcoin::Txid as BitcoinTxId;
use blockstack_lib::burnchains::Txid as StacksTxId;
use blockstack_lib::chainstate::stacks::StacksTransaction;

use crate::task::Task;

#[derive(Debug, Clone)]
pub enum Event {
    StacksTransactionUpdate(StacksTxId),
    BitcoinTransactionUpdate(StacksTxId),

    DepositSeen(Deposit),
    WithdrawalSeen,

    BitcoinBlock(Block),
    NextNonce(u64),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct State {}

pub fn update(state: State, event: Event) -> (State, Vec<Task>) {
    todo!();
}

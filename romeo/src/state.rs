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
    block_height: u64,
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

impl Deposit {
    fn process(&mut self, block_height: u64) -> Option<Task> {
        todo!();
    }
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

pub fn update(config: &Config, state: State, event: Event) -> (State, Vec<Task>) {
    match event {
        Event::StacksTransactionUpdate(_, _) => todo!(),
        Event::BitcoinTransactionUpdate(_, _) => todo!(),
        Event::BitcoinBlock(block) => process_bitcoin_block(config, state, block),
    }
}

fn process_bitcoin_block(config: &Config, mut state: State, block: Block) -> (State, Vec<Task>) {
    let deposits = parse_deposits(&block);
    let withdrawals = parse_withdrawals(&block);

    state.deposits.extend_from_slice(&deposits);
    state.withdrawals.extend_from_slice(&withdrawals);

    state.block_height = block
        .bip34_block_height()
        .expect("Failed to get block height");

    let task = Task::FetchBitcoinBlock(state.block_height + 1);

    let mint_tasks = state
        .deposits
        .iter_mut()
        .filter_map(|deposit| deposit.process(state.block_height));

    // TODO

    unimplemented!()
}

fn parse_deposits(block: &Block) -> Vec<Deposit> {
    // TODO
    vec![]
}

fn parse_withdrawals(block: &Block) -> Vec<Withdrawal> {
    // TODO
    vec![]
}

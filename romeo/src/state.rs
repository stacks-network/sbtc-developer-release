use bdk::bitcoin::{Block, Transaction as BitcoinTransaction, Txid as BitcoinTxId};
use blockstack_lib::burnchains::Txid as StacksTxId;
use blockstack_lib::chainstate::stacks::StacksTransaction;
use blockstack_lib::types::chainstate::StacksAddress;

use crate::config::Config;
use crate::event::Event;
use crate::event::TransactionStatus;
use crate::task::Task;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct State {
    deposits: Vec<Deposit>,
    withdrawals: Vec<Withdrawal>,
    next_stx_nonce: u64,
    block_height: u64,
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
    fn mint(&mut self, block_height: u64) -> Option<Task> {
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

impl Withdrawal {
    fn burn(&mut self, block_height: u64) -> Option<Task> {
        todo!();
    }

    fn fulfill(&mut self, block_height: u64) -> Option<Task> {
        todo!();
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Response<T> {
    tx: T,
    status: TransactionStatus,
}

pub fn update(config: &Config, state: State, event: Event) -> (State, Vec<Task>) {
    match event {
        Event::StacksTransactionUpdate(_, _) => todo!(),
        Event::BitcoinTransactionUpdate(txid, status) => {
            process_bitcoin_transaction(config, state, txid, status)
        }
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

    let mut tasks = vec![Task::FetchBitcoinBlock(state.block_height + 1)];

    let mint_tasks = state
        .deposits
        .iter_mut()
        .filter_map(|deposit| deposit.mint(state.block_height));

    let withdrawals = &mut state.withdrawals;

    let burn_tasks: Vec<_> = withdrawals
        .iter_mut()
        .filter_map(|withdrawal| withdrawal.burn(state.block_height))
        .collect();

    let fulfillment_tasks: Vec<_> = withdrawals
        .iter_mut()
        .filter_map(|withdrawal| withdrawal.fulfill(state.block_height))
        .collect();

    tasks.extend(mint_tasks);
    tasks.extend(burn_tasks);
    tasks.extend(fulfillment_tasks);

    (state, tasks)
}

fn parse_deposits(block: &Block) -> Vec<Deposit> {
    // TODO
    vec![]
}

fn parse_withdrawals(block: &Block) -> Vec<Withdrawal> {
    // TODO
    vec![]
}

fn process_bitcoin_transaction(
    config: &Config,
    mut state: State,
    txid: BitcoinTxId,
    status: TransactionStatus,
) -> (State, Vec<Task>) {
    //if let Some(fulfillment) = state.withdrawals.iter_mut().find_map(|withdrawal| {
    //withdrawal
    //.fulfillment
    //.filter(|fulfillment| fulfillment.tx.txid() == txid)
    //.as_mut()
    //}) {
    //fulfillment.status = status;
    //};

    // TODO: handle rejections and remove excess state on confirmations

    (state, vec![])
}

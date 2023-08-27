use bdk::bitcoin::{
    Address as BitcoinAddress, Block, Transaction as BitcoinTransaction, Txid as BitcoinTxId,
};
use blockstack_lib::burnchains::Txid as StacksTxId;
use blockstack_lib::chainstate::stacks::StacksTransaction;
use blockstack_lib::types::chainstate::StacksAddress;
use tracing::debug;

use crate::config::Config;
use crate::event::Event;
use crate::event::TransactionStatus;
use crate::task::Task;

/// The whole state of the application
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct State {
    contract: Option<Contract>,
    deposits: Vec<Deposit>,
    withdrawals: Vec<Withdrawal>,
    next_stx_nonce: u64,
    block_height: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Contract {
    txid: StacksTxId,
    status: TransactionStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Deposit {
    info: DepositInfo,
    mint: Option<Response<StacksTransaction>>,
    mint_pending: bool,
}

/// Relevant information for processing deposits
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DepositInfo {
    /// ID of the bitcoin deposit transaction
    pub txid: BitcoinTxId,

    /// Amount to deposit
    pub amount: u64,

    /// Recipient of the sBTC
    pub recipient: StacksAddress,

    /// Height of the Bitcoin blockchain where this tx is included
    pub block_height: u64,
}

impl Deposit {
    fn mint(&mut self, _block_height: u64) -> Option<Task> {
        todo!();
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Withdrawal {
    info: WithdrawalInfo,
    burn: Option<Response<StacksTransaction>>,
    fulfillment: Option<Response<BitcoinTransaction>>,
    burn_pending: bool,
    fulfillment_pending: bool,
}

/// Relevant information for processing withdrawals
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WithdrawalInfo {
    txid: BitcoinTxId,
    amount: u64,
    source: StacksAddress,
    recipient: BitcoinAddress,
    block_height: u64,
}

impl Withdrawal {
    fn burn(&mut self, _block_height: u64) -> Option<Task> {
        todo!();
    }

    fn fulfill(&mut self, _block_height: u64) -> Option<Task> {
        todo!();
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Response<T> {
    tx: T,
    status: TransactionStatus,
}

/// The beating heart of Romeo.
/// This function updates the system state in response to an I/O event.
/// It returns any new I/O tasks the system need to perform alongside the updated state.
#[tracing::instrument(skip(config, state))]
pub fn update(config: &Config, state: State, event: Event) -> (State, Vec<Task>) {
    debug!("Handling update");

    match event {
        Event::StacksTransactionUpdate(txid, status) => {
            process_stacks_transaction_update(config, state, txid, status)
        }
        Event::BitcoinTransactionUpdate(txid, status) => {
            process_bitcoin_transaction_update(config, state, txid, status)
        }
        Event::BitcoinBlock(block) => process_bitcoin_block(config, state, block),
        Event::AssetContractCreated(txid) => process_asset_contract_created(config, state, txid),
        event => panic!("Cannot handle yet: {:?}", event),
    }
}

fn process_bitcoin_block(_config: &Config, mut state: State, block: Block) -> (State, Vec<Task>) {
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

fn parse_deposits(_block: &Block) -> Vec<Deposit> {
    // TODO: #67
    vec![]
}

fn parse_withdrawals(_block: &Block) -> Vec<Withdrawal> {
    // TODO: #68
    vec![]
}

fn process_bitcoin_transaction_update(
    _config: &Config,
    state: State,
    _txid: BitcoinTxId,
    _status: TransactionStatus,
) -> (State, Vec<Task>) {
    // TODO: #67 and #68

    (state, vec![])
}

fn process_stacks_transaction_update(
    _config: &Config,
    state: State,
    _txid: StacksTxId,
    _status: TransactionStatus,
) -> (State, Vec<Task>) {
    // TODO: #73 and #68
    (state, vec![])
}

fn process_asset_contract_created(
    _config: &Config,
    mut state: State,
    txid: StacksTxId,
) -> (State, Vec<Task>) {
    // TODO: #73
    state.contract = Some(Contract {
        txid,
        status: TransactionStatus::Broadcasted,
    });

    let task = Task::CheckStacksTransactionStatus(txid);

    (state, vec![task])
}

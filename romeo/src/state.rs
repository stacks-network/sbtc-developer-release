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

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DepositInfo {
    pub txid: BitcoinTxId,
    pub amount: u64,
    pub recipient: StacksAddress,
    pub block_height: u64,
}

impl Deposit {
    fn mint(&mut self, block_height: u64) -> Option<Task> {
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WithdrawalInfo {
    txid: BitcoinTxId,
    amount: u64,
    source: StacksAddress,
    recipient: BitcoinAddress,
    block_height: u64,
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
    // TODO: #67
    vec![]
}

fn parse_withdrawals(block: &Block) -> Vec<Withdrawal> {
    // TODO: #68
    vec![]
}

fn process_bitcoin_transaction_update(
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

fn process_stacks_transaction_update(
    _config: &Config,
    mut state: State,
    _txid: StacksTxId,
    _status: TransactionStatus,
) -> (State, Vec<Task>) {
    // TODO
    (state, vec![])
}

fn process_asset_contract_created(
    _config: &Config,
    mut state: State,
    txid: StacksTxId,
) -> (State, Vec<Task>) {
    state.contract = Some(Contract {
        txid,
        status: TransactionStatus::Broadcasted,
    });

    let task = Task::CheckStacksTransactionStatus(txid);

    (state, vec![task])
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_process_bitcoin_block() {
        todo!();
    }

    #[test]
    fn test_parse_deposits() {
        todo!()
    }

    #[test]
    fn test_parse_withrawals() {
        todo!()
    }

    #[test]
    fn test_bitcoin_transaction_update() {
        todo!();
    }

    #[test]
    fn test_stacks_transaction_update() {
        todo!();
    }
}

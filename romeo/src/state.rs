//! State

use std::io::Cursor;

use bdk::bitcoin::{Address as BitcoinAddress, Block, Txid as BitcoinTxId};
use blockstack_lib::burnchains::Txid as StacksTxId;
use blockstack_lib::codec::StacksMessageCodec;
use blockstack_lib::types::chainstate::StacksAddress;
use blockstack_lib::vm::types::PrincipalData;
use sbtc_core::operations::op_return;
use stacks_core::codec::Codec;
use tracing::debug;

use crate::config::Config;
use crate::event::Event;
use crate::event::TransactionStatus;
use crate::task::Task;

/// The whole state of the application
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct State {
    contract: Option<TransactionRequest<StacksTxId>>,
    deposits: Vec<Deposit>,
    withdrawals: Vec<Withdrawal>,
    block_height: Option<u32>,
}

/// A parsed deposit
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Deposit {
    info: DepositInfo,
    mint: Option<TransactionRequest<StacksTxId>>,
}

/// A transaction request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TransactionRequest<T> {
    /// Created and passed on to a task
    Created,
    /// Acknowledged by a task with the status update
    Acknowledged(T, TransactionStatus),
}

/// Relevant information for processing deposits
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct DepositInfo {
    /// ID of the bitcoin deposit transaction
    pub txid: BitcoinTxId,

    /// Amount to deposit
    pub amount: u64,

    /// Recipient of the sBTC
    pub recipient: PrincipalData,

    /// Height of the Bitcoin blockchain where this deposit transaction exists
    pub block_height: u32,
}

impl Deposit {
    fn request_work(&mut self) -> Option<Task> {
        match self.mint {
            None => {
                self.mint = Some(TransactionRequest::Created);
                Some(Task::CreateMint(self.info.clone()))
            }
            Some(TransactionRequest::Created)
            | Some(TransactionRequest::Acknowledged(_, TransactionStatus::Confirmed)) => None,
            Some(TransactionRequest::Acknowledged(txid, TransactionStatus::Broadcasted)) => {
                Some(Task::CheckStacksTransactionStatus(txid))
            }
            Some(TransactionRequest::Acknowledged(txid, TransactionStatus::Rejected)) => {
                panic!("Mint transaction rejected: {}", txid)
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Withdrawal {
    info: WithdrawalInfo,
    burn: Option<TransactionRequest<StacksTxId>>,
    fulfillment: Option<TransactionRequest<BitcoinTxId>>,
}

/// Relevant information for processing withdrawals
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WithdrawalInfo {
    /// ID of the bitcoin withdrawal request transaction
    txid: BitcoinTxId,

    /// Amount to withdraw
    amount: u64,

    /// Where to withdraw sBTC from
    source: StacksAddress,

    /// Recipient of the BTC
    recipient: BitcoinAddress,

    /// Height of the Bitcoin blockchain where this withdrawal request
    /// transaction exists
    block_height: u32,
}

impl Withdrawal {
    fn burn(&mut self) -> Option<Task> {
        todo!();
    }

    fn fulfill(&mut self) -> Option<Task> {
        todo!();
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Response<T> {
    txid: T,
    status: TransactionStatus,
}

/// Spawn initial tasks given a recovered state
pub fn bootstrap(state: &State) -> Task {
    match state.contract {
        None => Task::CreateAssetContract,
        Some(_) => Task::FetchBitcoinBlock(get_next_block_height(state.block_height)),
    }
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
        Event::AssetContractBroadcasted(txid) => {
            process_asset_contract_created(config, state, txid)
        }
        Event::MintBroadcasted(deposit_info, txid) => {
            process_mint_created(state, deposit_info, txid)
        }
        event => panic!("Cannot handle yet: {:?}", event),
    }
}

fn process_bitcoin_block(config: &Config, mut state: State, block: Block) -> (State, Vec<Task>) {
    let deposits = parse_deposits(config, &block);
    let withdrawals = parse_withdrawals(&block);

    state.deposits.extend_from_slice(&deposits);
    state.withdrawals.extend_from_slice(&withdrawals);

    let new_block_height = block
        .bip34_block_height()
        .expect("Failed to get block height") as u32;

    state.block_height = Some(new_block_height);

    let mut tasks = create_transaction_status_update_requests(&mut state);
    tasks.push(Task::FetchBitcoinBlock(get_next_block_height(
        state.block_height,
    )));

    (state, tasks)
}

fn create_transaction_status_update_requests(state: &mut State) -> Vec<Task> {
    match state.contract {
        None | Some(TransactionRequest::Created) => vec![],
        Some(TransactionRequest::Acknowledged(txid, TransactionStatus::Broadcasted)) => {
            vec![Task::CheckStacksTransactionStatus(txid)]
        }
        Some(TransactionRequest::Acknowledged(_, TransactionStatus::Confirmed)) => {
            create_transaction_status_update_tasks(state)
        }
        Some(TransactionRequest::Acknowledged(txid, TransactionStatus::Rejected)) => {
            panic!("Contract creation transaction rejected: {}", txid)
        }
    }
}

fn create_transaction_status_update_tasks(state: &mut State) -> Vec<Task> {
    let mut tasks = vec![];

    let mint_tasks = state
        .deposits
        .iter_mut()
        .filter_map(|deposit| deposit.request_work());

    let burn_tasks: Vec<_> = state
        .withdrawals
        .iter_mut()
        .filter_map(|withdrawal| withdrawal.burn())
        .collect();

    let fulfillment_tasks: Vec<_> = state
        .withdrawals
        .iter_mut()
        .filter_map(|withdrawal| withdrawal.fulfill())
        .collect();

    tasks.extend(mint_tasks);
    tasks.extend(burn_tasks);
    tasks.extend(fulfillment_tasks);

    tasks
}

fn parse_deposits(config: &Config, block: &Block) -> Vec<Deposit> {
    let block_height = block
        .bip34_block_height()
        .expect("Failed to get block height") as u32;

    block
        .txdata
        .iter()
        .cloned()
        .filter_map(|tx| {
            let txid = tx.txid();

            op_return::deposit::Deposit::parse(config.private_key.network, tx)
                .ok()
                .map(|parsed_deposit| Deposit {
                    info: DepositInfo {
                        txid,
                        amount: parsed_deposit.amount,
                        recipient: convert_principal_data(parsed_deposit.recipient),
                        block_height,
                    },
                    mint: None,
                })
        })
        .collect()
}

fn convert_principal_data(data: stacks_core::utils::PrincipalData) -> PrincipalData {
    let bytes = data.serialize_to_vec();

    PrincipalData::consensus_deserialize(&mut Cursor::new(bytes)).unwrap()
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
    mut state: State,
    txid: StacksTxId,
    status: TransactionStatus,
) -> (State, Vec<Task>) {
    if status == TransactionStatus::Rejected {
        panic!("Stacks transaction failed");
    }

    let contract_response = state.contract.as_mut().into_iter();

    let deposit_responses = state
        .deposits
        .iter_mut()
        .filter_map(|deposit| deposit.mint.as_mut());

    let withdrawal_responses = state
        .withdrawals
        .iter_mut()
        .filter_map(|withdrawal| withdrawal.burn.as_mut());

    let statuses_updated: usize = std::iter::empty()
        .chain(contract_response)
        .chain(deposit_responses)
        .chain(withdrawal_responses)
        .map(|response| {
            let status_updated = match response {
                TransactionRequest::Acknowledged(current_txid, current_status) => {
                    if txid == *current_txid {
                        *current_status = status.clone();
                        true
                    } else {
                        false
                    }
                }
                TransactionRequest::Created => {
                    panic!("Got an update for a transaction that was not acknowledged")
                }
            };

            status_updated as usize
        })
        .sum();

    if statuses_updated != 1 {
        panic!(
            "Unexpected number of statuses updated: {}",
            statuses_updated
        );
    }

    let tasks = {
        let Some(TransactionRequest::Acknowledged(contract_txid, contract_status) ) = state.contract.as_ref() else  {
            panic!("Contract transaction should be acknowledged and broadcasted first");
        };

        if txid == *contract_txid
            && *contract_status == TransactionStatus::Broadcasted
            && state.block_height.is_none()
        {
            vec![Task::FetchBitcoinBlock(None)]
        } else {
            vec![]
        }
    };

    (state, tasks)
}

fn process_asset_contract_created(
    _config: &Config,
    mut state: State,
    txid: StacksTxId,
) -> (State, Vec<Task>) {
    state.contract = Some(TransactionRequest::Acknowledged(
        txid,
        TransactionStatus::Broadcasted,
    ));

    (state, vec![Task::CheckStacksTransactionStatus(txid)])
}

fn process_mint_created(
    mut state: State,
    deposit_info: DepositInfo,
    txid: StacksTxId,
) -> (State, Vec<Task>) {
    let deposit = state
        .deposits
        .iter_mut()
        .find(|deposit| deposit.info == deposit_info)
        .expect("Could not find a deposit for the mint");

    assert!(
        deposit.mint.is_none(),
        "Newly minted deposit already has a mint"
    );

    deposit.mint = Some(TransactionRequest::Acknowledged(
        txid,
        TransactionStatus::Broadcasted,
    ));

    (state, vec![])
}

fn get_next_block_height(height: Option<u32>) -> Option<u32> {
    height.map(|height| height + 1)
}

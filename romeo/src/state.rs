//! State

use std::io::Cursor;

use bdk::bitcoin::{Address as BitcoinAddress, Block, Txid as BitcoinTxId};
use blockstack_lib::{
	burnchains::Txid as StacksTxId, codec::StacksMessageCodec,
	types::chainstate::StacksAddress, vm::types::PrincipalData,
};
use sbtc_core::operations::op_return;
use stacks_core::codec::Codec;
use tracing::debug;

use crate::{
	config::Config,
	event::{Event, TransactionStatus},
	task::Task,
};

/// The whole state of the application
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct State {
	deposits: Vec<Deposit>,
	withdrawals: Vec<Withdrawal>,
	current_block_height: Option<u32>,
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
	Acknowledged {
		/// The transaction ID
		txid: T,
		/// The status of the transaction
		status: TransactionStatus,
		/// Whether the task has a pending request
		has_pending_task: bool,
	},
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
		match self.mint.as_mut() {
			None => {
				self.mint = Some(TransactionRequest::Created);
				Some(Task::CreateMint(self.info.clone()))
			}
			Some(TransactionRequest::Created)
			| Some(TransactionRequest::Acknowledged {
				status: TransactionStatus::Confirmed,
				..
			}) => None,
			Some(TransactionRequest::Acknowledged {
				txid,
				status: TransactionStatus::Broadcasted,
				has_pending_task,
			}) => {
				if !*has_pending_task {
					*has_pending_task = true;
					Some(Task::CheckStacksTransactionStatus(*txid))
				} else {
					None
				}
			}
			Some(TransactionRequest::Acknowledged {
				txid,
				status: TransactionStatus::Rejected,
				..
			}) => {
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
pub fn bootstrap(mut state: State) -> (State, Task) {
	// When bootstraping we're not expecting any transaction updates to come
	get_mut_transaction_requests(&mut state).for_each(|request| {
		if let TransactionRequest::Acknowledged {
			has_pending_task, ..
		} = request
		{
			*has_pending_task = false
		}
	});

	match state.current_block_height {
		None => (state, Task::GetContractBlockHeight),
		Some(height) => (state, Task::FetchBitcoinBlock(height + 1)),
	}
}

/// The beating heart of Romeo.
/// This function updates the system state in response to an I/O event.
/// It returns any new I/O tasks the system need to perform alongside the
/// updated state.
#[tracing::instrument(skip(config, state))]
pub fn update(
	config: &Config,
	state: State,
	event: Event,
) -> (State, Vec<Task>) {
	debug!("Handling update");

	match event {
		Event::ContractBlockHeight(height) => {
			process_contract_block_height(state, height)
		}
		Event::StacksTransactionUpdate(txid, status) => {
			process_stacks_transaction_update(config, state, txid, status)
		}
		Event::BitcoinTransactionUpdate(txid, status) => {
			process_bitcoin_transaction_update(config, state, txid, status)
		}
		Event::BitcoinBlock(height, block) => {
			process_bitcoin_block(config, state, height, block)
		}
		Event::MintBroadcasted(deposit_info, txid) => {
			process_mint_broadcasted(state, deposit_info, txid)
		}
		Event::BurnBroadcasted(_, _) => (state, vec![]),
		Event::FulfillBroadcasted(_, _) => (state, vec![]),
	}
}

fn process_contract_block_height(
	mut state: State,
	height: u32,
) -> (State, Vec<Task>) {
	match state.current_block_height {
		Some(current_height) => {
			panic!(
				"Got contract block height when state already contains it: {}",
				current_height
			)
		}
		None => {
			state.current_block_height = Some(height);

			(state, vec![Task::FetchBitcoinBlock(height + 1)])
		}
	}
}

fn process_bitcoin_block(
	config: &Config,
	mut state: State,
	height: u32,
	block: Block,
) -> (State, Vec<Task>) {
	let deposits = parse_deposits(config, height, &block);
	let withdrawals = parse_withdrawals(&block);

	state.deposits.extend_from_slice(&deposits);
	state.withdrawals.extend_from_slice(&withdrawals);

	state.current_block_height = Some(height);

	let mut tasks = create_transaction_status_update_tasks(&mut state);
	tasks.push(Task::FetchBitcoinBlock(height + 1));

	(state, tasks)
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

fn parse_deposits(config: &Config, height: u32, block: &Block) -> Vec<Deposit> {
	block
		.txdata
		.iter()
		.cloned()
		.filter_map(|tx| {
			let txid = tx.txid();

			op_return::deposit::Deposit::parse(
				config.bitcoin_credentials.network(),
				tx,
			)
			.ok()
			.map(|parsed_deposit| Deposit {
				info: DepositInfo {
					txid,
					amount: parsed_deposit.amount,
					recipient: convert_principal_data(parsed_deposit.recipient),
					block_height: height,
				},
				mint: None,
			})
		})
		.collect()
}

fn convert_principal_data(
	data: stacks_core::utils::PrincipalData,
) -> PrincipalData {
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
		panic!("Stacks transaction failed: {}", txid);
	}

	let statuses_updated: usize = get_mut_transaction_requests(&mut state)
        .map(|response| {
            let status_updated = match response {
                TransactionRequest::Acknowledged {
                    txid: current_txid,
                    status: current_status,
                    has_pending_task,
                } => {
                    if txid == *current_txid {
                        if !*has_pending_task {
                            panic!(
                                "Got the update {:?} for a transaction status update that doesn't have a pending task: {}", status, txid
                            );
                        }

                        *current_status = status.clone();
                        *has_pending_task = false;

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

	(state, vec![])
}

fn get_mut_transaction_requests(
	state: &mut State,
) -> impl Iterator<Item = &mut TransactionRequest<StacksTxId>> {
	let deposit_responses = state
		.deposits
		.iter_mut()
		.filter_map(|deposit| deposit.mint.as_mut());

	let withdrawal_responses = state
		.withdrawals
		.iter_mut()
		.filter_map(|withdrawal| withdrawal.burn.as_mut());

	deposit_responses.chain(withdrawal_responses)
}

fn process_mint_broadcasted(
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
		matches!(deposit.mint, Some(TransactionRequest::Created)),
		"Newly minted deposit already has a mint acknowledged"
	);

	deposit.mint = Some(TransactionRequest::Acknowledged {
		txid,
		status: TransactionStatus::Broadcasted,
		has_pending_task: false,
	});

	(state, vec![])
}

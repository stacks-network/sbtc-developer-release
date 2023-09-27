//! State

use std::io::Cursor;

use bdk::bitcoin::{Address as BitcoinAddress, Block, Txid as BitcoinTxId};
use blockstack_lib::{
	burnchains::Txid as StacksTxId, chainstate::stacks::StacksTransaction,
	codec::StacksMessageCodec, types::chainstate::StacksAddress,
	vm::types::PrincipalData,
};
use sbtc_core::operations::{
	op_return, op_return::withdrawal_request::WithdrawalRequestData,
};
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
	stacks_block_height: Option<u32>,
	bitcoin_block_height: Option<u32>,
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

	/// BTC recipient
	pub sbtc_wallet_address: BitcoinAddress,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Withdrawal {
	info: WithdrawalInfo,
	burn: Option<TransactionRequest<StacksTxId>>,
	fulfillment: Option<TransactionRequest<BitcoinTxId>>,
}

/// Relevant information for processing withdrawals
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct WithdrawalInfo {
	/// ID of the bitcoin withdrawal request transaction
	pub txid: BitcoinTxId,

	/// Amount to withdraw
	pub amount: u64,

	/// Where to withdraw sBTC from
	pub source: PrincipalData,

	/// Recipient of the BTC
	pub recipient: BitcoinAddress,

	/// Height of the Bitcoin blockchain where this withdrawal request
	/// transaction exists
	pub block_height: u32,

	/// BTC recipient
	pub sbtc_wallet_address: BitcoinAddress,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Response<T> {
	txid: T,
	status: TransactionStatus,
}

/// Spawn initial tasks given a recovered state
pub fn bootstrap(mut state: State) -> (State, Vec<Task>) {
	// When bootstraping we're not expecting any transaction updates to come
	get_mut_stacks_transaction_requests(&mut state).for_each(|request| {
		if let TransactionRequest::Acknowledged {
			has_pending_task, ..
		} = request
		{
			*has_pending_task = false
		}
	});

	let tasks = if state.stacks_block_height.is_none()
		|| state.bitcoin_block_height.is_none()
	{
		assert!(
			state.stacks_block_height.is_none()
				&& state.bitcoin_block_height.is_none(),
			"Only one of the block heights is missing"
		);

		vec![Task::GetContractBlockHeight]
	} else {
		vec![
			Task::FetchStacksBlock(state.stacks_block_height.unwrap() + 1),
			Task::FetchBitcoinBlock(state.bitcoin_block_height.unwrap() + 1),
		]
	};

	(state, tasks)
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
		Event::ContractBlockHeight(stacks_height, bitcoin_height) => {
			process_contract_block_height(state, stacks_height, bitcoin_height)
		}
		Event::StacksTransactionUpdate(txid, status) => {
			process_stacks_transaction_update(config, state, txid, status)
		}
		Event::BitcoinTransactionUpdate(txid, status) => {
			process_bitcoin_transaction_update(config, state, txid, status)
		}
		Event::StacksBlock(height, txs) => {
			process_stacks_block(config, state, height, txs)
		}
		Event::BitcoinBlock(height, block) => {
			process_bitcoin_block(config, state, height, block)
		}
		Event::MintBroadcasted(deposit_info, txid) => {
			process_mint_broadcasted(state, deposit_info, txid)
		}
		Event::BurnBroadcasted(withdrawal_info, txid) => {
			process_burn_broadcasted(state, withdrawal_info, txid)
		}
		Event::FulfillBroadcasted(withdrawal_info, txid) => {
			process_fulfillment_broadcasted(state, withdrawal_info, txid)
		}
	}
}

fn process_contract_block_height(
	mut state: State,
	stacks_height: u32,
	bitcoin_height: u32,
) -> (State, Vec<Task>) {
	let mut tasks = vec![];

	match state.stacks_block_height {
		Some(current_height) => {
			panic!(
				"Got contract block height when state already contains it: {}",
				current_height
			)
		}
		None => {
			state.stacks_block_height = Some(stacks_height);

			tasks.push(Task::FetchStacksBlock(stacks_height + 1));
		}
	};

	match state.bitcoin_block_height {
		Some(current_height) => {
			panic!(
                "Got contract bitcoin block height when state already contains it: {}",
                current_height
            )
		}
		None => {
			state.bitcoin_block_height = Some(bitcoin_height);

			tasks.push(Task::FetchBitcoinBlock(bitcoin_height + 1));
		}
	};

	(state, tasks)
}

fn process_stacks_block(
	_config: &Config,
	mut state: State,
	stacks_height: u32,
	_txs: Vec<StacksTransaction>,
) -> (State, Vec<Task>) {
	state.stacks_block_height = Some(stacks_height);

	let mut tasks = vec![Task::FetchStacksBlock(stacks_height + 1)];
	tasks.extend(get_stacks_status_checks(&mut state));
	tasks.extend(get_bitcoin_transactions(&mut state));

	(state, tasks)
}

fn process_bitcoin_block(
	config: &Config,
	mut state: State,
	bitcoin_height: u32,
	block: Block,
) -> (State, Vec<Task>) {
	state.deposits.extend(
		parse_deposits(config, bitcoin_height, &block)
			.into_iter()
			.filter(|deposit| {
				deposit.info.sbtc_wallet_address == config.sbtc_wallet_address()
			})
			.collect::<Vec<_>>(),
	);
	state.withdrawals.extend(
		parse_withdrawals(config, &block)
			.into_iter()
			.filter(|withdrawal| {
				withdrawal.info.sbtc_wallet_address
					== config.sbtc_wallet_address()
			})
			.collect::<Vec<_>>(),
	);
	state.bitcoin_block_height = Some(bitcoin_height);

	let mut tasks = vec![Task::FetchBitcoinBlock(bitcoin_height + 1)];
	tasks.extend(get_bitcoin_status_checks(&mut state));
	tasks.extend(get_stacks_transactions(&mut state));

	(state, tasks)
}

fn get_stacks_transactions(state: &mut State) -> Vec<Task> {
	let deposit_tasks = state.deposits.iter_mut().filter_map(|deposit| {
		match deposit.mint.as_mut() {
			None => {
				deposit.mint = Some(TransactionRequest::Created);
				Some(Task::CreateMint(deposit.info.clone()))
			}
			_ => None,
		}
	});

	let withdrawal_tasks =
		state
			.withdrawals
			.iter_mut()
			.filter_map(|withdrawal| match withdrawal.burn.as_mut() {
				None => {
					withdrawal.burn = Some(TransactionRequest::Created);
					Some(Task::CreateBurn(withdrawal.info.clone()))
				}
				_ => None,
			});

	deposit_tasks.chain(withdrawal_tasks).collect()
}

fn get_bitcoin_transactions(state: &mut State) -> Vec<Task> {
	state
		.withdrawals
		.iter_mut()
		.filter_map(|withdrawal| match withdrawal.fulfillment.as_mut() {
			None => {
				withdrawal.fulfillment = Some(TransactionRequest::Created);
				Some(Task::CreateFulfillment(withdrawal.info.clone()))
			}
			_ => None,
		})
		.collect()
}

fn get_stacks_status_checks(state: &mut State) -> Vec<Task> {
	let deposit_tasks = state.deposits.iter_mut().filter_map(|deposit| {
		match deposit.mint.as_mut() {
			Some(TransactionRequest::Acknowledged {
				txid,
				status: TransactionStatus::Broadcasted,
				has_pending_task,
			}) if !*has_pending_task => {
				*has_pending_task = true;
				Some(Task::CheckStacksTransactionStatus(*txid))
			}
			_ => None,
		}
	});

	let withdrawal_tasks =
		state
			.withdrawals
			.iter_mut()
			.filter_map(|withdrawal| match withdrawal.burn.as_mut() {
				Some(TransactionRequest::Acknowledged {
					txid,
					status: TransactionStatus::Broadcasted,
					has_pending_task,
				}) if !*has_pending_task => {
					*has_pending_task = true;
					Some(Task::CheckStacksTransactionStatus(*txid))
				}
				_ => None,
			});

	deposit_tasks.chain(withdrawal_tasks).collect()
}

fn get_bitcoin_status_checks(state: &mut State) -> Vec<Task> {
	state
		.withdrawals
		.iter_mut()
		.filter_map(|withdrawal| match withdrawal.fulfillment.as_mut() {
			Some(TransactionRequest::Acknowledged {
				txid,
				status: TransactionStatus::Broadcasted,
				has_pending_task,
			}) if !*has_pending_task => {
				*has_pending_task = true;
				Some(Task::CheckBitcoinTransactionStatus(*txid))
			}
			_ => None,
		})
		.collect()
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
					sbtc_wallet_address: parsed_deposit.sbtc_wallet_address,
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

fn parse_withdrawals(config: &Config, block: &Block) -> Vec<Withdrawal> {
	let block_height = block
		.bip34_block_height()
		.expect("Failed to get block height") as u32;

	block
		.txdata
		.iter()
		.cloned()
		.filter_map(|tx| {
			let txid = tx.txid();

			op_return::withdrawal_request::try_parse_withdrawal_request(
				config.bitcoin_network,
				tx,
			)
			.ok()
			.map(
				|WithdrawalRequestData {
				     payee_bitcoin_address,
				     drawee_stacks_address,
				     amount,
				     peg_wallet,
				     ..
				 }| {
					let blockstack_lib_address =
						StacksAddress::consensus_deserialize(&mut Cursor::new(
							drawee_stacks_address.serialize_to_vec(),
						))
						.unwrap();
					let source = PrincipalData::from(blockstack_lib_address);

					Withdrawal {
						info: WithdrawalInfo {
							txid,
							amount,
							source,
							recipient: payee_bitcoin_address,
							block_height,
							sbtc_wallet_address: peg_wallet,
						},
						burn: None,
						fulfillment: None,
					}
				},
			)
		})
		.collect()
}

fn process_bitcoin_transaction_update(
	_config: &Config,
	mut state: State,
	txid: BitcoinTxId,
	status: TransactionStatus,
) -> (State, Vec<Task>) {
	if status == TransactionStatus::Rejected {
		panic!("Stacks transaction failed: {}", txid);
	}

	let statuses_updated: usize = state
        .withdrawals
        .iter_mut()
        .map(|withdrawal| match withdrawal.fulfillment.as_mut() {
            Some(TransactionRequest::Acknowledged {
                txid: current_txid,
                status: current_status,
                has_pending_task,
            }) if *current_txid == txid => {
                if !*has_pending_task {
                    panic!(
                        "Got the update {:?} for a Stacks transaction that doesn't have a pending task: {}", status, txid
                    );
                }

                *current_status = status.clone();
                *has_pending_task = false;

                true
            }
            _ => false,
        }).map(|updated| updated as usize).sum();

	if statuses_updated != 1 {
		panic!(
			"Unexpected number of statuses updated: {}",
			statuses_updated
		);
	}

	let tasks = get_stacks_transactions(&mut state);

	(state, tasks)
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

	let statuses_updated: usize = get_mut_stacks_transaction_requests(&mut state)
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
                                "Got the update {:?} for a Stacks transaction that doesn't have a pending task: {}", status, txid
                            );
                        }

                        *current_status = status.clone();
                        *has_pending_task = false;

                        true
                    } else {
                        false
                    }
                }
                _ => false
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

	let tasks = get_bitcoin_transactions(&mut state);

	(state, tasks)
}

fn get_mut_stacks_transaction_requests(
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

fn process_burn_broadcasted(
	mut state: State,
	withdrawal_info: WithdrawalInfo,
	txid: StacksTxId,
) -> (State, Vec<Task>) {
	let withdrawal = state
		.withdrawals
		.iter_mut()
		.find(|withdrawal| withdrawal.info == withdrawal_info)
		.expect("Could not find a withdrawal for the burn");

	assert!(
		matches!(withdrawal.burn, Some(TransactionRequest::Created)),
		"Newly burned withdrawal already has a burn acknowledged"
	);

	withdrawal.burn = Some(TransactionRequest::Acknowledged {
		txid,
		status: TransactionStatus::Broadcasted,
		has_pending_task: false,
	});

	(state, vec![])
}

fn process_fulfillment_broadcasted(
	mut state: State,
	withdrawal_info: WithdrawalInfo,
	txid: BitcoinTxId,
) -> (State, Vec<Task>) {
	let withdrawal = state
		.withdrawals
		.iter_mut()
		.find(|withdrawal| withdrawal.info == withdrawal_info)
		.expect("Could not find a withdrawal for the fulfillment");

	assert!(
		matches!(withdrawal.fulfillment, Some(TransactionRequest::Created)),
		"Newly fulfilled withdrawal already has a fulfillment acknowledged"
	);

	withdrawal.fulfillment = Some(TransactionRequest::Acknowledged {
		txid,
		status: TransactionStatus::Broadcasted,
		has_pending_task: false,
	});

	(state, vec![])
}

//! State

use std::io::Cursor;
pub mod transaction_request;

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
use transaction_request::{Acknowledged, TransactionRequest};

use crate::{
	config::Config,
	event::{Event, TransactionStatus},
	task::Task,
};

/// The delay in blocks between receiving a deposit request and creating
/// the deposit transaction.
const STX_TRANSACTION_DELAY_BLOCKS: u32 = 1;

/// Romeo internal state
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum State {
	/// Starting state without any data
	Uninitialized,

	/// Contract detected and block heights known
	ContractDetected {
		/// Stacks block height
		stacks_block_height: u32,
		/// Bitcoin block height
		bitcoin_block_height: u32,
	},

	/// Contract public key setup transaction broadcasted
	ContractPublicKeySetup {
		/// Stacks block height
		stacks_block_height: u32,
		/// Bitcoin block height
		bitcoin_block_height: u32,
		/// Set public key transaction request
		public_key_setup: TransactionRequest<StacksTxId>,
	},

	/// State initialized and ready to process deposits and withdrawals
	Initialized {
		/// Stacks block height
		stacks_block_height: u32,
		/// Bitcoin block height
		bitcoin_block_height: u32,
		/// Deposits
		deposits: Vec<Deposit>,
		/// Withdrawals
		withdrawals: Vec<Withdrawal>,
	},
}

impl State {
	/// Creates uninitialized state
	pub fn new() -> Self {
		Default::default()
	}

	/// Spawn initial tasks given a recovered state
	pub fn bootstrap(&mut self) -> Vec<Task> {
		match self {
			State::Uninitialized => vec![Task::GetContractBlockHeight],
			State::ContractDetected { .. } => {
				vec![Task::UpdateContractPublicKey]
			}
			State::ContractPublicKeySetup {
				stacks_block_height,
				..
			} => {
				vec![Task::FetchStacksBlock(*stacks_block_height + 1)]
			}
			State::Initialized {
				stacks_block_height,
				bitcoin_block_height,
				deposits,
				withdrawals,
			} => {
				deposits
					.iter_mut()
					.filter_map(|deposit| deposit.mint.as_mut())
					.chain(
						withdrawals
							.iter_mut()
							.filter_map(|withdrawal| withdrawal.burn.as_mut()),
					)
					.for_each(|req| {
						if let TransactionRequest::Acknowledged(
							Acknowledged {
								has_pending_task, ..
							},
						) = req
						{
							*has_pending_task = false;
						}
					});

				withdrawals
					.iter_mut()
					.filter_map(|withdrawal| withdrawal.fulfillment.as_mut())
					.for_each(|req| {
						if let TransactionRequest::Acknowledged(
							Acknowledged {
								has_pending_task, ..
							},
						) = req
						{
							*has_pending_task = false;
						}
					});

				vec![
					Task::FetchStacksBlock(*stacks_block_height + 1),
					Task::FetchBitcoinBlock(*bitcoin_block_height + 1),
				]
			}
		}
	}

	/// Updates the state and return new tasks to be scheduled
	#[tracing::instrument(skip(self, config))]
	pub fn update(&mut self, event: Event, config: &Config) -> Vec<Task> {
		match event {
			Event::ContractBlockHeight(stacks_height, bitcoin_height) => self
				.process_contract_block_height(stacks_height, bitcoin_height),
			Event::ContractPublicKeySetBroadcasted(txid) => {
				self.process_set_contract_public_key(txid)
			}
			Event::StacksTransactionUpdate(txid, status) => self
				.process_stacks_transaction_update(txid, status, config.strict),
			Event::BitcoinTransactionUpdate(txid, status) => self
				.process_bitcoin_transaction_update(
					txid,
					status,
					config.strict,
				),
			Event::StacksBlock(height, txs) => {
				self.process_stacks_block(height, txs)
			}
			Event::BitcoinBlock(height, block) => {
				self.process_bitcoin_block(config, height, block)
			}
			Event::MintBroadcasted(deposit_info, txid) => {
				self.process_mint_broadcasted(deposit_info, txid, config);
				vec![]
			}
			Event::BurnBroadcasted(withdrawal_info, txid) => {
				self.process_burn_broadcasted(withdrawal_info, txid, config);
				vec![]
			}
			Event::FulfillBroadcasted(withdrawal_info, txid) => {
				self.process_fulfillment_broadcasted(
					withdrawal_info,
					txid,
					config,
				);
				vec![]
			}
		}
	}

	fn process_contract_block_height(
		&mut self,
		contract_stacks_block_height: u32,
		contract_bitcoin_block_height: u32,
	) -> Vec<Task> {
		assert!(
			matches!(self, State::Uninitialized),
			"Cannot process contract block height when state is initialized"
		);

		*self = State::ContractDetected {
			stacks_block_height: contract_stacks_block_height,
			bitcoin_block_height: contract_bitcoin_block_height,
		};

		vec![Task::UpdateContractPublicKey]
	}

	fn process_set_contract_public_key(
		&mut self,
		txid: StacksTxId,
	) -> Vec<Task> {
		let State::ContractDetected {
			stacks_block_height,
			bitcoin_block_height,
		} = self
		else {
			panic!("Cannot process contract public key when contract is not detected")
		};

		let stacks_block_height = *stacks_block_height;
		let bitcoin_block_height = *bitcoin_block_height;

		*self = State::ContractPublicKeySetup {
			stacks_block_height,
			bitcoin_block_height,
			public_key_setup: TransactionRequest::Acknowledged(Acknowledged {
				txid,
				status: TransactionStatus::Broadcasted,
				has_pending_task: false,
			}),
		};

		vec![Task::FetchStacksBlock(stacks_block_height + 1)]
	}

	fn process_stacks_transaction_update(
		&mut self,
		txid: StacksTxId,
		status: TransactionStatus,
		strict: bool,
	) -> Vec<Task> {
		let mut tasks = self.get_bitcoin_transactions();

		let statuses_updated = match self {
			State::Uninitialized => None,
			State::ContractDetected { .. } => None,
			State::ContractPublicKeySetup {
				stacks_block_height,
				bitcoin_block_height,
				public_key_setup,
			} => {
				tracing::debug!("Filtering stacks' pubkey set");
				if let Some(Acknowledged {
					status: current_status,
					has_pending_task,
					..
				}) = public_key_setup
					.filtered_acknowledged_ref_mut(txid, strict, &status)
					.and_then(|res| res.ok())
				{
					tracing::debug!("Stacks txn {txid} update");

					*current_status = status.clone();
					*has_pending_task = false;

					if *current_status == TransactionStatus::Confirmed {
						let bitcoin_block_height = *bitcoin_block_height;

						*self = Self::Initialized {
							stacks_block_height: *stacks_block_height,
							bitcoin_block_height,
							deposits: vec![],
							withdrawals: vec![],
						};

						tasks.push(Task::FetchBitcoinBlock(
							bitcoin_block_height + 1,
						));
					}
					Some(1)
				} else {
					Some(0)
				}
			}
			State::Initialized {
				deposits,
				withdrawals,
				..
			} => {
				let statuses_updated: usize = deposits
					.iter_mut()
					.filter_map(|deposit| deposit.mint.as_mut())
					.chain(
						withdrawals
							.iter_mut()
							.filter_map(|withdrawal| withdrawal.burn.as_mut()),
					)
					.filter_map(|req| {
						tracing::debug!("Filtering stacks txn");
						req.filtered_acknowledged_ref_mut(txid, strict, &status)
							.and_then(|r| r.ok())
					})
					.map(
						|Acknowledged {
						     status: current_status,
						     has_pending_task,
						     ..
						 }| {
							tracing::debug!("Stacks txn {txid} update");

							*current_status = status.clone();
							*has_pending_task = false;

							1
						},
					)
					.sum();

				Some(statuses_updated)
			}
		};

		if let Some(statuses_updated) = statuses_updated {
			if statuses_updated != 1 {
				panic!(
					"Unexpected number of Stacks statuses updated: {}",
					statuses_updated
				);
			}
		}

		tasks
	}

	fn process_bitcoin_transaction_update(
		&mut self,
		txid: BitcoinTxId,
		status: TransactionStatus,
		strict: bool,
	) -> Vec<Task> {
		let State::Initialized { withdrawals, .. } = self else {
			panic!("Cannot process Bitcoin transaction update when state is not initialized");
		};

		if status == TransactionStatus::Rejected {
			if strict {
				panic!("Bitcoin transaction failed: {}", txid);
			} else {
				debug!("Bitcoin transaction failed: {}", txid);
			}
		}

		let statuses_updated: usize = withdrawals
			.iter_mut()
			.filter_map(|withdrawal| {
				withdrawal
					.fulfillment
					.as_mut()
					.and_then(|req| {
						tracing::debug!("Filtering btc txn");
						req.filtered_acknowledged_ref_mut(txid, strict, &status)
					})
					.and_then(|ack| ack.ok())
			})
			.map(|ack| {
				tracing::debug!("btc txn {txid} update");
				ack.status = status.clone();
				ack.has_pending_task = false;
				1
			})
			.sum();

		if statuses_updated != 1 {
			panic!(
				"Unexpected number of statuses updated: {}",
				statuses_updated
			);
		}

		self.get_stacks_transactions()
	}

	fn process_stacks_block(
		&mut self,
		stacks_height: u32,
		_txs: Vec<StacksTransaction>,
	) -> Vec<Task> {
		let stacks_block_height = match self {
			State::Uninitialized | State::ContractDetected { .. } => panic!("Cannot process Stacks block if uninitialized or contract detected"),
			State::ContractPublicKeySetup {
				stacks_block_height,
				..
			} => stacks_block_height,
			State::Initialized {
				stacks_block_height,
				..
			} => stacks_block_height,
		};

		*stacks_block_height = stacks_height;

		let mut tasks = vec![Task::FetchStacksBlock(stacks_height + 1)];

		tasks.extend(self.get_stacks_status_checks());
		tasks.extend(self.get_bitcoin_transactions());

		tasks
	}

	fn process_bitcoin_block(
		&mut self,
		config: &Config,
		bitcoin_height: u32,
		block: Block,
	) -> Vec<Task> {
		let State::Initialized {
			bitcoin_block_height,
			deposits,
			withdrawals,
			..
		} = self
		else {
			panic!("Cannot process Stacks block if not initialized")
		};

		*bitcoin_block_height = bitcoin_height;

		deposits.extend(parse_deposits(config, bitcoin_height, &block));
		withdrawals.extend(parse_withdrawals(config, &block));

		let mut tasks = vec![Task::FetchBitcoinBlock(bitcoin_height + 1)];

		tasks.extend(self.get_bitcoin_status_checks());
		tasks.extend(self.get_stacks_transactions());

		tasks
	}

	fn get_bitcoin_transactions(&mut self) -> Vec<Task> {
		let State::Initialized { withdrawals, .. } = self else {
			return vec![];
		};

		withdrawals
			.iter_mut()
			.filter_map(|withdrawal| match withdrawal.burn {
				Some(TransactionRequest::Acknowledged(Acknowledged {
					status: TransactionStatus::Confirmed,
					..
				})) => match withdrawal.fulfillment.as_mut() {
					None => {
						withdrawal.fulfillment =
							Some(TransactionRequest::Created);
						Some(Task::CreateFulfillment(withdrawal.info.clone()))
					}
					_ => None,
				},
				_ => None,
			})
			.collect()
	}

	fn get_stacks_transactions(&mut self) -> Vec<Task> {
		match self {
			State::Uninitialized | State::ContractPublicKeySetup { .. } => {
				vec![]
			}
			State::ContractDetected { .. } => {
				vec![Task::UpdateContractPublicKey]
			}

			State::Initialized {
				deposits,
				withdrawals,
				stacks_block_height,
				..
			} => {
				let deposit_tasks = deposits.iter_mut().filter_map(|deposit| {
					match deposit.mint.as_mut() {
						None => {
							// We often receive the deposit before the
							// transaction is actually mined. By scheduling the
							// transaction for a block later than the current
							// one we make ourselves resilient to mining delays
							// without complex logic.
							let scheduled_block_height = *stacks_block_height
								+ STX_TRANSACTION_DELAY_BLOCKS;

							deposit.mint =
								Some(TransactionRequest::Scheduled {
									block_height: scheduled_block_height,
								});

							debug!("Scheduled deposit {} for minting on stacks block height {}.",
								deposit.info.txid, scheduled_block_height);

							None
						}
						Some(TransactionRequest::Scheduled {
							block_height,
						}) if (*block_height <= *stacks_block_height) => {
							// Only initiate the mint task if the current
							// stacks block is or is after the stacks block
							// for which the mint is scheduled.
							deposit.mint = Some(TransactionRequest::Created);
							debug!("Created mint for {}.", deposit.info.txid);
							Some(Task::CreateMint(deposit.info.clone()))
						}
						_ => None,
					}
				});

				let withdrawal_tasks =
					withdrawals.iter_mut().filter_map(|withdrawal| {
						match withdrawal.burn.as_mut() {
							None => {
								let scheduled_block_height =
									*stacks_block_height
										+ STX_TRANSACTION_DELAY_BLOCKS;

								withdrawal.burn =
									Some(TransactionRequest::Scheduled {
										block_height: scheduled_block_height,
									});

								debug!("Scheduled withdrawal {} for minting on stacks block height {}.",
									withdrawal.info.txid, scheduled_block_height);

								None
							}
							Some(TransactionRequest::Scheduled {
								block_height,
							}) if (*block_height <= *stacks_block_height) => {
								// Only initiate the mint task if the current
								// stacks block is or is after the stacks block
								// for which the mint is scheduled.
								withdrawal.burn =
									Some(TransactionRequest::Created);
								debug!(
									"Created burn for {}.",
									withdrawal.info.txid
								);
								Some(Task::CreateBurn(withdrawal.info.clone()))
							}
							_ => None,
						}
					});

				deposit_tasks.chain(withdrawal_tasks).collect()
			}
		}
	}

	fn get_stacks_status_checks(&mut self) -> Vec<Task> {
		let reqs = match self {
			State::Uninitialized | State::ContractDetected { .. } => vec![],
			State::ContractPublicKeySetup {
				public_key_setup, ..
			} => vec![public_key_setup],
			State::Initialized {
				deposits,
				withdrawals,
				..
			} => {
				let mint_reqs = deposits
					.iter_mut()
					.filter_map(|deposit| deposit.mint.as_mut());
				let burn_reqs = withdrawals
					.iter_mut()
					.filter_map(|withdrawal| withdrawal.burn.as_mut());

				mint_reqs.chain(burn_reqs).collect()
			}
		};

		reqs.into_iter()
			.filter_map(|req| match req {
				TransactionRequest::Acknowledged(Acknowledged {
					txid,
					status: TransactionStatus::Broadcasted,
					has_pending_task,
				}) if !*has_pending_task => {
					*has_pending_task = true;
					Some(Task::CheckStacksTransactionStatus(*txid))
				}
				_ => None,
			})
			.collect()
	}

	fn get_bitcoin_status_checks(&mut self) -> Vec<Task> {
		match self {
			State::Initialized { withdrawals, .. } => withdrawals
				.iter_mut()
				.filter_map(|withdrawal| withdrawal.fulfillment.as_mut())
				.filter_map(|req| match req {
					TransactionRequest::Acknowledged(Acknowledged {
						txid,
						status: TransactionStatus::Broadcasted,
						has_pending_task,
					}) if !*has_pending_task => {
						*has_pending_task = true;
						Some(Task::CheckBitcoinTransactionStatus(*txid))
					}
					_ => None,
				})
				.collect(),
			_ => vec![],
		}
	}

	fn process_mint_broadcasted(
		&mut self,
		deposit_info: DepositInfo,
		txid: StacksTxId,
		config: &Config,
	) {
		let State::Initialized { deposits, .. } = self else {
			panic!("Cannot process broadcasted mint if uninitialized")
		};

		let deposit = deposits
			.iter_mut()
			.find(|deposit| deposit.info == deposit_info)
			.expect("Could not find a deposit for the mint");

		debug!("Mint broadcasted: {:?}", deposit.mint);
		if config.strict {
			assert!(
				matches!(deposit.mint, Some(TransactionRequest::Created)),
				"Newly minted deposit already has mint acknowledged"
			);
		}

		deposit.mint = Some(TransactionRequest::Acknowledged(Acknowledged {
			txid,
			status: TransactionStatus::Broadcasted,
			has_pending_task: false,
		}));
	}

	fn process_burn_broadcasted(
		&mut self,
		withdrawal_info: WithdrawalInfo,
		txid: StacksTxId,
		config: &Config,
	) {
		let State::Initialized { withdrawals, .. } = self else {
			panic!("Cannot process broadcasted burn if uninitialized")
		};

		let withdrawal = withdrawals
			.iter_mut()
			.find(|withdrawal| withdrawal.info == withdrawal_info)
			.expect("Could not find a withdrawal for the burn");

		if config.strict {
			assert!(
				matches!(withdrawal.burn, Some(TransactionRequest::Created)),
				"Newly burned withdrawal already has burn acknowledged"
			);
		}

		withdrawal.burn =
			Some(TransactionRequest::Acknowledged(Acknowledged {
				txid,
				status: TransactionStatus::Broadcasted,
				has_pending_task: false,
			}));
	}

	fn process_fulfillment_broadcasted(
		&mut self,
		withdrawal_info: WithdrawalInfo,
		txid: BitcoinTxId,
		config: &Config,
	) {
		let State::Initialized { withdrawals, .. } = self else {
			panic!("Cannot process broadcasted fulfillment if uninitialized")
		};

		let withdrawal = withdrawals
			.iter_mut()
			.find(|withdrawal| withdrawal.info == withdrawal_info)
			.expect("Could not find a withdrawal for the fulfillment");

		if config.strict {
			assert!(
			matches!(withdrawal.fulfillment, Some(TransactionRequest::Created)),
			"Newly fulfilled withdrawal already has fulfillment acknowledged"
		);
		}

		withdrawal.fulfillment =
			Some(TransactionRequest::Acknowledged(Acknowledged {
				txid,
				status: TransactionStatus::Broadcasted,
				has_pending_task: false,
			}));
	}
}

impl Default for State {
	fn default() -> Self {
		Self::Uninitialized
	}
}

fn parse_deposits(
	config: &Config,
	bitcoin_height: u32,
	block: &Block,
) -> Vec<Deposit> {
	let sbtc_wallet_address = config.sbtc_wallet_address();
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
			.filter(|parsed_deposit| {
				parsed_deposit.sbtc_wallet_address == sbtc_wallet_address
			})
			.map(|parsed_deposit| {
				let bytes = parsed_deposit.recipient.serialize_to_vec();
				let recipient = PrincipalData::consensus_deserialize(
					&mut Cursor::new(bytes),
				)
				.unwrap();

				Deposit {
					info: DepositInfo {
						txid,
						amount: parsed_deposit.amount,
						recipient,
						block_height: bitcoin_height,
					},
					mint: None,
				}
			})
		})
		.collect()
}

fn parse_withdrawals(config: &Config, block: &Block) -> Vec<Withdrawal> {
	let sbtc_wallet_address = config.sbtc_wallet_address();
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
			.filter(|parsed_withdrawal| {
				parsed_withdrawal.sbtc_wallet == sbtc_wallet_address
			})
			.map(
				|WithdrawalRequestData {
				     payee_bitcoin_address,
				     drawee_stacks_address,
				     amount,
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
						},
						burn: None,
						fulfillment: None,
					}
				},
			)
		})
		.collect()
}

/// A parsed deposit
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Deposit {
	info: DepositInfo,
	mint: Option<TransactionRequest<StacksTxId>>,
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

/// A parsed withdrawal
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Withdrawal {
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
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use assert_matches::assert_matches;
	use bdk::bitcoin::hashes::Hash;

	use super::*;

	#[test]
	fn process_stacks_transaction_update_positive_public_key_setup() {
		let tx_req =
			TransactionRequest::<StacksTxId>::Acknowledged(Acknowledged {
				txid: StacksTxId::from_sighash_bytes(&[0; 32]),
				status: TransactionStatus::Broadcasted,
				has_pending_task: true,
			});
		let mut state = State::ContractPublicKeySetup {
			stacks_block_height: 1,
			bitcoin_block_height: 100,
			public_key_setup: tx_req,
		};
		assert_matches!(
			state
				.process_stacks_transaction_update(
					StacksTxId::from_sighash_bytes(&[0; 32]),
					TransactionStatus::Confirmed,
					true,
				)
				.first()
				.unwrap(),
			Task::FetchBitcoinBlock(101)
		);
	}

	#[test]
	fn process_stacks_transaction_update_initilized() {
		let txid = StacksTxId::from_sighash_bytes(&[0; 32]);
		let tx_req =
			TransactionRequest::<StacksTxId>::Acknowledged(Acknowledged {
				txid,
				status: TransactionStatus::Broadcasted,
				has_pending_task: true,
			});

		let d = Deposit {
			info: DepositInfo {
				txid: BitcoinTxId::from_slice([0; 32].as_slice()).unwrap(),
				amount: 0,
				recipient: PrincipalData::parse(
					"ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM",
				)
				.unwrap(),
				block_height: 100,
			},
			mint: Some(tx_req),
		};

		let mut state = State::Initialized {
			stacks_block_height: 1,
			bitcoin_block_height: 100,
			deposits: vec![d],
			withdrawals: vec![],
		};

		assert!(state
			.process_stacks_transaction_update(
				txid,
				TransactionStatus::Confirmed,
				true,
			)
			.is_empty());

		assert_matches!(
			state,
			State::Initialized {
				deposits,
				..
			} => {
				assert_matches!(
					deposits.first().unwrap().mint,
					Some(TransactionRequest::Acknowledged(Acknowledged {
						has_pending_task: false,
						status: TransactionStatus::Confirmed,
						..
					}))
				)
			}
		);
	}

	#[test]
	fn process_bitcoin_transaction_update_initilized() {
		let txid = StacksTxId::from_sighash_bytes(&[0; 32]);
		let bitcoin_txid = BitcoinTxId::from_slice([0; 32].as_slice()).unwrap();

		let w = Withdrawal {
			info: WithdrawalInfo {
				txid: bitcoin_txid,
				amount: 0,
				block_height: 100,
				source: PrincipalData::parse(
					"ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM",
				)
				.unwrap(),
				recipient: BitcoinAddress::from_str(
					"bcrt1q3tj2fr9scwmcw3rq5m6jslva65f2rqjxfrjz47",
				)
				.unwrap(),
			},
			burn: Some(TransactionRequest::<StacksTxId>::Acknowledged(
				Acknowledged {
					txid,
					status: TransactionStatus::Broadcasted,
					has_pending_task: true,
				},
			)),
			fulfillment: Some(TransactionRequest::<BitcoinTxId>::Acknowledged(
				Acknowledged {
					txid: bitcoin_txid,
					status: TransactionStatus::Broadcasted,
					has_pending_task: true,
				},
			)),
		};

		let mut state = State::Initialized {
			stacks_block_height: 1,
			bitcoin_block_height: 100,
			deposits: vec![],
			withdrawals: vec![w],
		};

		assert!(state
			.process_bitcoin_transaction_update(
				bitcoin_txid,
				TransactionStatus::Confirmed,
				true,
			)
			.is_empty());

		assert_matches!(
			state,
			State::Initialized {
				withdrawals,
				..
			} => {
				assert_matches!(
					withdrawals.first().unwrap().fulfillment,
					Some(TransactionRequest::Acknowledged(Acknowledged {
						has_pending_task: false,
						status: TransactionStatus::Confirmed,
						..
					}))
				)
			}
		);
	}
}

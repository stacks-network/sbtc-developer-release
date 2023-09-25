//! Event

use bdk::bitcoin::{Block, Txid as BitcoinTxId};
use blockstack_lib::burnchains::Txid as StacksTxId;

use crate::state::{DepositInfo, WithdrawalInfo};

/// Events are spawned from tasks and used
/// to update the system state.
#[derive(
	Clone, serde::Serialize, serde::Deserialize, derivative::Derivative,
)]
#[derivative(Debug)]
pub enum Event {
	/// Block height of the contract deployment transaction
	ContractBlockHeight(u32),

	/// A mint transaction has been created and broadcasted
	MintBroadcasted(DepositInfo, StacksTxId),

	/// A burn transaction has been created and broadcasted
	BurnBroadcasted(WithdrawalInfo, StacksTxId),

	/// A fulfill transaction has been created and broadcasted
	FulfillBroadcasted(WithdrawalInfo, BitcoinTxId),

	/// A stacks node has responded with an updated status regarding this txid
	StacksTransactionUpdate(StacksTxId, TransactionStatus),

	/// A bitcoin node has responded with an updated status regarding this txid
	BitcoinTransactionUpdate(BitcoinTxId, TransactionStatus),

	/// A wild bitcoin block has appeared
	BitcoinBlock(u32, #[derivative(Debug = "ignore")] Block),
}

/// Status of a broadcasted transaction, useful for implementing retry logic
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum TransactionStatus {
	/// Broadcasted to a node
	Broadcasted,
	/// This transaction has received
	/// `Config::number_of_required_confirmations` confirmations
	Confirmed,
	/// There are indications that this transaction will never be mined
	Rejected,
}

//! Task

use bdk::bitcoin::Txid as BitcoinTxId;
use blockstack_lib::burnchains::Txid as StacksTxId;

use crate::state;

/// Represents I/O operations performed by the system
#[derive(Debug)]
pub enum Task {
	/// Get the block height of the contract deployment
	GetContractBlockHeight,

	/// Updates the contract public key
	UpdateContractPublicKey,

	/// Create and broadcast a mint stacks transaction
	CreateMint(state::DepositInfo),

	/// Create and broadcast a burn stacks transaction
	CreateBurn(state::WithdrawalInfo),

	/// Create and broadcast a fulfill bitcoin transaction
	CreateFulfillment(state::WithdrawalInfo),

	/// Poll a bitcoin node for the status of a transaction
	CheckBitcoinTransactionStatus(BitcoinTxId),

	/// Poll a stacks node for the status of a transaction
	CheckStacksTransactionStatus(StacksTxId),

	/// Fetch a Stacks block for the given block height
	FetchStacksBlock(u32),

	/// Fetch a Bitcoin block for the given block height
	FetchBitcoinBlock(u32),
}

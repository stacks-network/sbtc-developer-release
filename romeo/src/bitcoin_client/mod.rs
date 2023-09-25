//! Bitcoin client implementations

use std::fmt::Debug;

use async_trait::async_trait;
use bdk::bitcoin::{Block, Transaction, Txid};

use crate::event::TransactionStatus;

pub mod esplora;
pub mod rpc;

/// Bitcoin client
#[async_trait]
pub trait BitcoinClient: Send + Sync + Debug {
	/// Broadcast a bitcoin transaction
	async fn broadcast(&self, tx: Transaction) -> anyhow::Result<()>;

	/// Get the status of a transaction
	async fn get_tx_status(
		&self,
		txid: Txid,
	) -> anyhow::Result<TransactionStatus>;

	/// Fetch a block at the given block height, waiting if needed
	async fn fetch_block(
		&self,
		block_height: u32,
	) -> anyhow::Result<(u32, Block)>;

	/// Get the current block height
	async fn get_height(&self) -> anyhow::Result<u32>;

	/// Sign relevant inputs of a bitcoin transaction
	async fn sign(&self, _tx: Transaction) -> anyhow::Result<Transaction> {
		// TODO #68
		todo!()
	}
}

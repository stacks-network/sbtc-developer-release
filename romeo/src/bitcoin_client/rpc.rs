//! RPC Bitcoin client

use std::time::Duration;

use anyhow::anyhow;
use async_trait::async_trait;
use bdk::{
	bitcoin::{Block, Transaction, Txid},
	bitcoincore_rpc::{self, Auth, Client, RpcApi},
};
use tokio::{task::spawn_blocking, time::sleep};
use tracing::trace;
use url::Url;

use crate::{bitcoin_client::BitcoinClient, event::TransactionStatus};

const BLOCK_POLLING_INTERVAL: Duration = Duration::from_secs(5);

/// Bitcoin RPC client
#[derive(Debug, Clone)]
pub struct RPCClient(Url);

impl RPCClient {
	/// Create a new RPC client
	pub fn new(url: Url) -> anyhow::Result<Self> {
		let username = url.username().to_string();
		let password = url.password().unwrap_or_default().to_string();

		if username.is_empty() {
			return Err(anyhow::anyhow!("Username is empty"));
		}

		if password.is_empty() {
			return Err(anyhow::anyhow!("Password is empty"));
		}

		Ok(Self(url))
	}

	async fn execute<F, T>(
		&self,
		f: F,
	) -> anyhow::Result<bitcoincore_rpc::Result<T>>
	where
		F: FnOnce(Client) -> bitcoincore_rpc::Result<T> + Send + 'static,
		T: Send + 'static,
	{
		let mut url = self.0.clone();

		let username = url.username().to_string();
		let password = url.password().unwrap_or_default().to_string();

		url.set_username("").unwrap();
		url.set_password(None).unwrap();

		let client =
			Client::new(url.as_ref(), Auth::UserPass(username, password))?;

		Ok(spawn_blocking(move || f(client)).await?)
	}
}

#[async_trait]
impl BitcoinClient for RPCClient {
	async fn broadcast(&self, tx: Transaction) -> anyhow::Result<()> {
		self.execute(move |client| client.send_raw_transaction(&tx))
			.await??;

		Ok(())
	}

	async fn get_tx_status(
		&self,
		txid: Txid,
	) -> anyhow::Result<TransactionStatus> {
		let tx = self
			.execute(move |client| client.get_raw_transaction_info(&txid, None))
			.await??;

		if tx.blockhash.is_some() {
			Ok(TransactionStatus::Confirmed)
		} else {
			Ok(TransactionStatus::Broadcasted)
		}
	}

	async fn fetch_block(
		&self,
		block_height: u32,
	) -> anyhow::Result<(u32, Block)> {
		let block_hash = loop {
			let res = self
				.execute(move |client| {
					client.get_block_hash(block_height as u64)
				})
				.await?;

			match res {
				Ok(hash) => {
					trace!("Got block hash: {}", hash);
					break hash;
				}
				Err(bitcoincore_rpc::Error::JsonRpc(
					bitcoincore_rpc::jsonrpc::Error::Rpc(err),
				)) => {
					if err.code == -8 {
						trace!("Block not found, retrying...");
					} else {
						Err(anyhow!("Error fetching block: {:?}", err))?;
					}
				}
				Err(err) => Err(anyhow!("Error fetching block: {:?}", err))?,
			};

			sleep(BLOCK_POLLING_INTERVAL).await;
		};

		let block = self
			.execute(move |client| client.get_block(&block_hash))
			.await??;

		Ok((block_height, block))
	}

	async fn get_height(&self) -> anyhow::Result<u32> {
		let info = self
			.execute(|client| client.get_blockchain_info())
			.await??;

		Ok(info.blocks as u32)
	}
}

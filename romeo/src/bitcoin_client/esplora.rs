//! Bitcoin client

use std::time::Duration;

use async_trait::async_trait;
use bdk::{
	bitcoin::{self, Transaction, Txid},
	esplora_client::{self, AsyncClient, Builder},
};
use futures::Future;
use tracing::trace;

use super::BitcoinClient;
use crate::event::{self, TransactionStatus};

const BLOCK_POLLING_INTERVAL: Duration = Duration::from_secs(5);

/// Facilitates communication with a Bitcoin esplora server
#[derive(Debug, Clone)]
pub struct EsploraClient(AsyncClient);

impl EsploraClient {
	/// Create Esplora Bitcoin client
	pub fn new(url: impl AsRef<str>) -> anyhow::Result<Self> {
		Ok(Self(Builder::new(url.as_ref()).build_async()?))
	}
}

#[async_trait]
impl BitcoinClient for EsploraClient {
	async fn broadcast(&self, tx: Transaction) -> anyhow::Result<()> {
		retry(|| self.0.broadcast(&tx)).await
	}

	async fn get_tx_status(
		&self,
		txid: Txid,
	) -> anyhow::Result<TransactionStatus> {
		let status = retry(|| self.0.get_tx_status(&txid)).await?;

		Ok(match status {
			Some(esplora_client::TxStatus {
				confirmed: true, ..
			}) => event::TransactionStatus::Confirmed,
			Some(esplora_client::TxStatus {
				confirmed: false, ..
			}) => event::TransactionStatus::Broadcasted,
			None => event::TransactionStatus::Rejected,
		})
	}

	#[tracing::instrument(skip(self))]
	async fn fetch_block(
		&self,
		block_height: u32,
	) -> anyhow::Result<(u32, bitcoin::Block)> {
		let mut current_height = retry(|| self.0.get_height()).await?;

		trace!("Looking for block height: {}", current_height + 1);
		while current_height < block_height {
			tokio::time::sleep(BLOCK_POLLING_INTERVAL).await;
			current_height = retry(|| self.0.get_height()).await?;
		}

		let block_summaries =
			retry(|| self.0.get_blocks(Some(block_height))).await?;
		let block_summary = block_summaries.first().ok_or_else(|| {
			anyhow::anyhow!("Could not find block at given block height")
		})?;

		let block = retry(|| self.0.get_block_by_hash(&block_summary.id))
			.await?
			.ok_or_else(|| {
				anyhow::anyhow!("Found no block for the given block hash")
			})?;

		trace!("Fetched block");

		Ok((block_height, block))
	}

	async fn get_height(&self) -> anyhow::Result<u32> {
		retry(|| self.0.get_height()).await
	}
}

async fn retry<T, O, Fut>(operation: O) -> anyhow::Result<T>
where
	O: Clone + Fn() -> Fut,
	Fut: Future<Output = Result<T, bdk::esplora_client::Error>>,
{
	let operation = || async {
		operation.clone()().await.map_err(|err| match err {
			esplora_client::Error::Reqwest(_) => {
				backoff::Error::transient(anyhow::anyhow!(err))
			}
			err => backoff::Error::permanent(anyhow::anyhow!(err)),
		})
	};

	let notify = |err, duration| {
		trace!("Retrying in {:?} after error: {:?}", duration, err);
	};

	backoff::future::retry_notify(
		backoff::ExponentialBackoff::default(),
		operation,
		notify,
	)
	.await
}

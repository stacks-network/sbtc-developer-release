//! Bitcoin client

use std::time::Duration;

use async_trait::async_trait;
use bdk::bitcoin;
use bdk::bitcoin::schnorr;
use bdk::bitcoin::secp256k1;
use bdk::esplora_client;
use futures::Future;
use tracing::trace;

use crate::event;

use super::client::BitcoinClient;

const BLOCK_POLLING_INTERVAL: Duration = Duration::from_secs(5);

/// Facilitates communication with a Bitcoin esplora server
#[derive(Debug, Clone)]
pub struct EsploraClient {
    client: esplora_client::AsyncClient,
    private_key: bitcoin::PrivateKey,
}

impl EsploraClient {
    /// Construct a new bitcoin client
    pub fn new(esplora_url: &str, private_key: bitcoin::PrivateKey) -> anyhow::Result<Self> {
        let client = esplora_client::Builder::new(esplora_url).build_async()?;

        Ok(Self {
            client,
            private_key,
        })
    }

    /// Broadcast a bitcoin transaction
    pub async fn broadcast(&self, tx: &bitcoin::Transaction) -> anyhow::Result<()> {
        retry(|| self.client.broadcast(tx)).await
    }

    /// Sign relevant inputs of a bitcoin transaction
    pub async fn sign(&self, _tx: bitcoin::Transaction) -> anyhow::Result<bitcoin::Transaction> {
        // TODO #68
        todo!()
    }

    /// Bitcoin taproot address associated with the private key
    pub async fn taproot_address(&self) -> bitcoin::Address {
        let secp = secp256k1::Secp256k1::new();
        let internal_key: schnorr::UntweakedPublicKey =
            self.private_key.public_key(&secp).inner.into();

        bitcoin::Address::p2tr(&secp, internal_key, None, self.private_key.network)
    }
}

#[async_trait]
impl BitcoinClient for EsploraClient {
    /// Get the status of a transaction
    async fn get_tx_status(
        &mut self,
        txid: &bitcoin::Txid,
    ) -> anyhow::Result<event::TransactionStatus> {
        let status = retry(|| self.client.get_tx_status(txid)).await?;

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

    /// Fetch a block at the given block height.
    /// If the current block height is lower than the requested block height
    /// this function will poll the blockchain until that height is reached.
    #[tracing::instrument(skip(self))]
    async fn fetch_block(&mut self, block_height: u32) -> anyhow::Result<bitcoin::Block> {
        let mut current_height = retry(|| self.client.get_height()).await?;

        trace!("Looking for block height: {}", current_height + 1);
        while current_height < block_height {
            tokio::time::sleep(BLOCK_POLLING_INTERVAL).await;
            current_height = retry(|| self.client.get_height()).await?;
        }

        let block_summaries = retry(|| self.client.get_blocks(Some(block_height))).await?;
        let block_summary = block_summaries
            .first()
            .ok_or_else(|| anyhow::anyhow!("Could not find block at given block height"))?;

        let block = retry(|| self.client.get_block_by_hash(&block_summary.id))
            .await?
            .ok_or_else(|| anyhow::anyhow!("Found no block for the given block hash"))?;

        trace!("Fetched block");

        Ok(block)
    }

    /// Get the current height of the Bitcoin chain
    async fn get_height(&mut self) -> anyhow::Result<u32> {
        retry(|| self.client.get_height()).await
    }
}

async fn retry<T, O, Fut>(operation: O) -> anyhow::Result<T>
where
    O: Clone + Fn() -> Fut,
    Fut: Future<Output = Result<T, bdk::esplora_client::Error>>,
{
    let operation = || async {
        operation.clone()().await.map_err(|err| match err {
            esplora_client::Error::Reqwest(_) => backoff::Error::transient(anyhow::anyhow!(err)),
            err => backoff::Error::permanent(anyhow::anyhow!(err)),
        })
    };

    let notify = |err, duration| {
        trace!("Retrying in {:?} after error: {:?}", duration, err);
    };

    backoff::future::retry_notify(backoff::ExponentialBackoff::default(), operation, notify).await
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    use super::*;

    // These integration tests are for exploration/experimentation but should be removed once we have more decent tests
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[ignore]
    async fn get_block() {
        let config =
            Config::from_path("./testing/config.json").expect("Failed to find config file");

        let mut bitcoin_client =
            EsploraClient::new(config.bitcoin_node_url.as_str(), config.private_key).unwrap();

        let block_height = bitcoin_client.get_height().await.unwrap();
        let block = bitcoin_client.fetch_block(block_height).await.unwrap();

        println!("Block: {:?}", block);

        assert!(block.txdata.len() > 10);
    }
}

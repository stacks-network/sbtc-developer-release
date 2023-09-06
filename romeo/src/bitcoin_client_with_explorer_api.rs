use anyhow::anyhow;
use bdk::bitcoin;
use bdk::bitcoin::schnorr;
use bdk::bitcoin::secp256k1;
use bdk::bitcoin::Block;
use reqwest::{Client, Request, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tracing::trace;

use crate::event;

pub struct TxDetails {
    pub confirmations: u32,
}

pub struct BitcoinExplorerApiClient {
    client: Client,
    rest_url: String,
    private_key: bitcoin::PrivateKey,
}

impl BitcoinExplorerApiClient {
    pub fn new(rest_url: &str, private_key: bitcoin::PrivateKey) -> anyhow::Result<Self> {
        let client = reqwest::Client::new();

        Ok(Self {
            client,
            rest_url: rest_url.to_string(),
            private_key,
        })
    }

    async fn send_request<T>(&mut self, req: Request) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
    {
        let request_url = req.url().to_string();
        let res = self.client.execute(req).await?;

        if res.status() == StatusCode::OK {
            Ok(res.json::<T>().await?)
        } else {
            let details = res.json::<Value>().await?;

            trace!(
                "Request failure details: {:?}",
                serde_json::to_string(&details)?
            );

            Err(anyhow!(format!(
                "Request not 200: {}: {}",
                request_url, details["error"]
            )))
        }
    }

    /// Get the status of a transaction
    pub async fn get_tx_status(
        &self,
        txid: &bitcoin::Txid,
    ) -> anyhow::Result<event::TransactionStatus> {
        let tx_details: anyhow::Result<TxDetails> = self
            .send_request(
                self.client
                    .get(&format!("{}/api/tx/{}", self.rest_url, txid.to_string()))
                    .build()?,
            )
            .await?;

        Ok(match tx_details {
            Ok(TxDetails { confirmations }) => {
                if confirmations > 0 {
                    event::TransactionStatus::Confirmed
                } else {
                    event::TransactionStatus::Broadcasted
                }
            }
            Err(_) => event::TransactionStatus::Rejected,
        })
    }

    /// Fetch a block at the given block height.
    /// If the current block height is lower than the requested block height
    /// this function will poll the blockchain until that height is reached.
    #[tracing::instrument(skip(self))]
    pub async fn fetch_block(&self, block_height: u32) -> anyhow::Result<bitcoin::Block> {
        let mut current_height = self.get_height().await?;

        while current_height < block_height {
            tracing::debug!("Polling: {}", current_height);
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            current_height = self.get_height().await?;
        }

        let block = self.get_block_by_height(block_height).await?;

        tracing::debug!("Fetched block");
        Ok(block)
    }

    /// Get the current height of the Bitcoin chain
    pub async fn get_height(&self) -> anyhow::Result<u32> {
        let response: Result<u32, anyhow::Error> = self
            .send_request(
                self.client
                    .get(&format!("{}/api/blocks/tip", self.rest_url))
                    .build()?,
            )
            .await?;
        response
    }

    /// Get the current height of the Bitcoin chain
    pub async fn get_block_by_height(&self, height: u32) -> anyhow::Result<Block> {
        let response: Result<Block, anyhow::Error> = self
            .send_request(
                self.client
                    .get(&format!("{}/api/block/{}", self.rest_url, height))
                    .build()?,
            )
            .await?;
        response
    }
    /// Bitcoin taproot address associated with the private key
    pub async fn taproot_address(&self) -> bitcoin::Address {
        let secp = secp256k1::Secp256k1::new();
        let internal_key: schnorr::UntweakedPublicKey =
            self.private_key.public_key(&secp).inner.into();

        bitcoin::Address::p2tr(&secp, internal_key, None, self.private_key.network)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    use super::*;

    // These integration tests are for exploration/experimentation but should be removed once we have more decent tests
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[ignore]
    async fn get_height() {
        let config =
            Config::from_path("./testing/config.json").expect("Failed to find config file");

        let bitcoin_client =
            BitcoinExplorerApiClient::new(config.bitcoin_node_url.as_str(), config.private_key)
                .unwrap();

        let block_height = bitcoin_client.get_height().await.unwrap();
        let block = bitcoin_client.fetch_block(block_height).await.unwrap();

        println!("Block: {:?}", block);

        assert!(block.txdata.len() > 10);
    }
}

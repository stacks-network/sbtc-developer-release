//! Stacks client

use anyhow::anyhow;
use blockstack_lib::codec::StacksMessageCodec;
use blockstack_lib::core::CHAIN_ID_TESTNET;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use reqwest::{Request, StatusCode};
use serde_json::Value;
use std::io::Cursor;
use std::slice::from_raw_parts_mut;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use tracing::trace;

use serde::de::DeserializeOwned;

use blockstack_lib::burnchains::Txid as StacksTxId;
use blockstack_lib::chainstate::stacks::{
    StacksTransaction, StacksTransactionSigner, TransactionAnchorMode, TransactionPostConditionMode,
};

use crate::config::Config;
use crate::event::TransactionStatus;

/// Wrapped Stacks Client which can be shared safely between threads.
#[derive(Clone, Debug)]
pub struct LockedClient(Arc<Mutex<StacksClient>>);

impl LockedClient {
    /// Lock and obtain a handle to the inner stacks client
    pub async fn lock(&self) -> MutexGuard<StacksClient> {
        self.0.lock().await
    }
}

impl From<StacksClient> for LockedClient {
    fn from(client: StacksClient) -> Self {
        Self(Arc::new(Mutex::new(client)))
    }
}

/// Stateful client for creating and broadcasting Stacks transactions
///
/// This client keeps track of the last executed nonce for the given
/// key.
#[derive(Debug)]
pub struct StacksClient {
    config: Config,
    http_client: reqwest::Client,
}

impl StacksClient {
    /// Create a new StacksClient
    pub fn new(config: Config, http_client: reqwest::Client) -> Self {
        Self {
            config,
            http_client,
        }
    }

    async fn send_request<T>(&mut self, req: Request) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
    {
        let request_url = req.url().to_string();
        let res = self.http_client.execute(req).await?;

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

    /// Sign and broadcast an unsigned stacks transaction
    pub async fn sign_and_broadcast(
        &mut self,
        mut tx: StacksTransaction,
    ) -> anyhow::Result<StacksTxId> {
        tx.set_origin_nonce(self.get_nonce_info().await?.possible_next_nonce);
        tx.set_tx_fee(self.calculate_fee(tx.tx_len()).await?);

        tx.anchor_mode = TransactionAnchorMode::Any;
        tx.post_condition_mode = TransactionPostConditionMode::Allow;
        tx.chain_id = CHAIN_ID_TESTNET;

        let mut signer = StacksTransactionSigner::new(&tx);

        signer
            .sign_origin(&self.config.stacks_private_key())
            .unwrap();

        tx = signer.get_tx().unwrap();

        let mut tx_bytes = vec![];
        tx.consensus_serialize(&mut tx_bytes).unwrap();

        let res = self
            .send_request(
                self.http_client
                    .post(self.transaction_url())
                    .header("Content-type", "application/octet-stream")
                    .body(tx_bytes)
                    .build()?,
            )
            .await?;

        Ok(res)
    }

    /// Get transaction status for a given txid
    pub async fn get_transation_status(
        &mut self,
        txid: StacksTxId,
    ) -> anyhow::Result<TransactionStatus> {
        let res: Value = self
            .send_request(
                self.http_client
                    .get(self.get_transation_details_url(txid))
                    .header("Accept", "application/json")
                    .build()?,
            )
            .await?;

        let tx_status_str = res["tx_status"]
            .as_str()
            .expect("Could not get raw transaction from response");

        Ok(match tx_status_str {
            "pending" => TransactionStatus::Broadcasted,
            "success" => TransactionStatus::Confirmed,
            "abort_by_response" => TransactionStatus::Rejected,
            status => panic!("Unknown transation status: {}", status),
        })
    }

    async fn get_nonce_info(&mut self) -> anyhow::Result<NonceInfo> {
        Ok(self
            .http_client
            .get(self.nonce_url())
            .send()
            .await?
            .json()
            .await?)
    }

    async fn calculate_fee(&self, tx_len: u64) -> anyhow::Result<u64> {
        let fee_rate: u64 = self
            .http_client
            .get(self.fee_url())
            .send()
            .await?
            .json()
            .await?;

        Ok(fee_rate * tx_len)
    }

    fn transaction_url(&self) -> reqwest::Url {
        self.config
            .stacks_node_url
            .join("/v2/transactions")
            .unwrap()
    }

    fn get_transation_details_url(&self, txid: StacksTxId) -> reqwest::Url {
        self.config
            .stacks_node_url
            .join(&format!("/extended/v1/tx/{}", txid))
            .unwrap()
    }

    fn nonce_url(&self) -> reqwest::Url {
        let mut rng = thread_rng();
        let random_string: String = (0..16).map(|_| rng.sample(Alphanumeric) as char).collect();

        // We need to make sure node returns the uncached nonce, so we add a cachebuster
        let path = format!(
            "/extended/v1/address/{}/nonces?cachebuster={}",
            self.config.stacks_address(),
            random_string
        );

        self.config.stacks_node_url.join(&path).unwrap()
    }

    fn fee_url(&self) -> reqwest::Url {
        self.config
            .stacks_node_url
            .join("/v2/fees/transfer")
            .unwrap()
    }
}

#[derive(serde::Deserialize)]
struct NonceInfo {
    possible_next_nonce: u64,
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    use super::*;

    // These integration tests are for exploration/experimentation but should be removed once we have more decent tests
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[ignore]
    async fn get_nonce_info() {
        let config =
            Config::from_path("./testing/config.json").expect("Failed to find config file");
        let http_client = reqwest::Client::new();

        let mut stacks_client = StacksClient::new(config, http_client);

        let nonce_info = stacks_client.get_nonce_info().await.unwrap();
        assert_eq!(nonce_info.possible_next_nonce, 122);

        assert!(true);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[ignore]
    async fn get_fee_rate() {
        let config =
            Config::from_path("./testing/config.json").expect("Failed to find config file");
        let http_client = reqwest::Client::new();

        let stacks_client = StacksClient::new(config, http_client);

        stacks_client.calculate_fee(123).await.unwrap();
    }
}

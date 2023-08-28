//! Stacks client

use bdk::bitcoin::{util::uint::Uint128, Network};
use blockstack_lib::codec::StacksMessageCodec;
use blockstack_lib::core::CHAIN_ID_TESTNET;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

use serde::de::Error;
use serde::Deserialize;

use blockstack_lib::burnchains::Txid as StacksTxId;
use blockstack_lib::chainstate::stacks::{
    StacksTransaction, StacksTransactionSigner, TransactionAnchorMode, TransactionPostConditionMode,
};
use blockstack_lib::{
    address::{
        AddressHashMode, C32_ADDRESS_VERSION_MAINNET_SINGLESIG,
        C32_ADDRESS_VERSION_TESTNET_SINGLESIG,
    },
    types::chainstate::{StacksAddress, StacksPrivateKey, StacksPublicKey},
};

/// Wrapped Stacks Client which can be shared safely between threads.
#[derive(Clone, Debug)]
pub struct LockedClient(Arc<Mutex<StacksClient>>);

impl LockedClient {
    /// Lock and obtain a handle to the inner stacks client
    pub async fn lock<'a>(&'a self) -> MutexGuard<'a, StacksClient> {
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
    private_key: StacksPrivateKey,
    stacks_node_url: reqwest::Url,
    http_client: reqwest::Client,
    network: Network,
}

impl StacksClient {
    /// Create a new StacksClient
    pub async fn new(
        private_key: StacksPrivateKey,
        stacks_node_url: reqwest::Url,
        http_client: reqwest::Client,
        network: Network,
    ) -> anyhow::Result<Self> {
        let mut self_ = Self {
            private_key,
            stacks_node_url,
            http_client,
            network,
        };

        Ok(self_)
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

        let mut signer = StacksTransactionSigner::new(&mut tx);

        signer.sign_origin(&self.private_key).unwrap();

        tx = signer.get_tx().unwrap();

        let mut tx_bytes = vec![];
        tx.consensus_serialize(&mut tx_bytes).unwrap();

        let res = self
            .http_client
            .post(self.transaction_url())
            .header("Content-type", "application/octet-stream")
            .body(tx_bytes)
            .send()
            .await?;

        Ok(res.json().await?)
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
        self.stacks_node_url.join("/v2/transactions").unwrap()
    }

    fn nonce_url(&self) -> reqwest::Url {
        let mut rng = thread_rng();
        let random_string: String = (0..16).map(|_| rng.sample(Alphanumeric) as char).collect();

        // We need to make sure node returns the uncached nonce, so we add a cachebuster
        let path = format!(
            "/extended/v1/address/{}/nonces?cachebuster={}",
            self.stx_address(),
            random_string
        );

        self.stacks_node_url.join(&path).unwrap()
    }

    fn fee_url(&self) -> reqwest::Url {
        self.stacks_node_url.join("/v2/fees/transfer").unwrap()
    }

    fn stx_address(&self) -> StacksAddress {
        let address_version = self.address_version();
        let hash_mode = AddressHashMode::SerializeP2PKH;

        StacksAddress::from_public_keys(address_version, &hash_mode, 1, &vec![self.public_key()])
            .unwrap()
    }

    fn public_key(&self) -> StacksPublicKey {
        let mut public_key = StacksPublicKey::from_private(&self.private_key);
        public_key.set_compressed(true);
        public_key
    }

    fn address_version(&self) -> u8 {
        match self.network {
            Network::Bitcoin => C32_ADDRESS_VERSION_MAINNET_SINGLESIG,
            Network::Testnet => C32_ADDRESS_VERSION_TESTNET_SINGLESIG,
            _ => panic!("Unsupported network"),
        }
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

    // Hacky integration test. TODO: Make it more proper
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[ignore]
    async fn get_account_info() {
        let config =
            Config::from_path("./testing/config.json").expect("Failed to find config file");
        let http_client = reqwest::Client::new();

        let mut stacks_client = StacksClient::new(
            config.stacks_private_key(),
            config.stacks_node_url,
            http_client,
            config.private_key.network,
        )
        .await
        .unwrap();

        assert_eq!(stacks_client.nonce, 122);

        let nonce_info = stacks_client.get_nonce_info().await.unwrap();

        assert_eq!(nonce_info.possible_next_nonce, 122);

        assert!(true);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[ignore]
    async fn get_fee() {
        let config =
            Config::from_path("./testing/config.json").expect("Failed to find config file");
        let http_client = reqwest::Client::new();

        let mut stacks_client = StacksClient::new(
            config.stacks_private_key(),
            config.stacks_node_url,
            http_client,
            config.private_key.network,
        )
        .await
        .unwrap();

        stacks_client.calculate_fee(123).await;
    }
}

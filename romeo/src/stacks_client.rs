//! Stacks client

use bdk::bitcoin::{Network, util::uint::Uint128};
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

use serde::Deserialize;
use serde::de::Error;

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
        todo!();
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
    nonce: u64,
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
            nonce: 0,
        };

        self_.reconcile_nonce().await?;

        Ok(self_)
    }

    async fn reconcile_nonce(&mut self) -> anyhow::Result<()> {
        let account_info = self.get_account_info().await?;
        self.nonce = account_info.nonce;

        Ok(())
    }

    async fn get_account_info(&mut self) -> anyhow::Result<AccountInfo> {
        Ok(self
            .http_client
            .get(self.account_url())
            .send()
            .await?
            .json()
            .await?)
    }

    fn account_url(&self) -> reqwest::Url {
        let path = format!("/v2/accounts/{}", self.stx_address());
        self.stacks_node_url.join(&path).unwrap()
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
struct AccountInfo {
    #[serde(deserialize_with = "deserialize_balance")]
    balance: u64,
    nonce: u64,
}

fn deserialize_balance<'de, D>(deserializer: D) -> Result<u64, D::Error>
where D: serde::Deserializer<'de> {
    let buf = String::deserialize(deserializer)?;
    let bytes = hex::decode(buf.trim_start_matches("0x")).map_err(D::Error::custom)?;

    let val = Uint128::from_be_slice(&bytes).map_err(D::Error::custom)?;
    Ok(val.low_u64())
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    use super::*;

    // Hacky integration test. TODO: Make it more proper
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[ignore]
    async fn get_account_info() {
        let config = Config::from_path("./testing/config.json").expect("Failed to find config file");
        let http_client = reqwest::Client::new();
        
        let mut stacks_client = StacksClient::new(config.stacks_private_key(), config.stacks_node_url, http_client, config.private_key.network).await.unwrap();

        let account_info = stacks_client.get_account_info().await.unwrap();

        assert_eq!(account_info.nonce, 121);
        assert_eq!(account_info.balance, 482115874);
        
        assert!(true);
    }
}

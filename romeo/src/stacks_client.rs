//! Stacks client

use std::sync::Arc;

use anyhow::anyhow;
use blockstack_lib::{
	burnchains::Txid as StacksTxId,
	chainstate::stacks::{
		StacksTransaction, StacksTransactionSigner, TransactionAnchorMode,
		TransactionPostConditionMode,
	},
	codec::StacksMessageCodec,
	core::CHAIN_ID_TESTNET,
	types::chainstate::StacksPrivateKey,
	vm::{
		types::{QualifiedContractIdentifier, StandardPrincipalData},
		ContractName,
	},
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest::Request;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::sync::{Mutex, MutexGuard};
use tracing::debug;

use crate::{config::Config, event::TransactionStatus};

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
		let status = res.status();
		let body = res.text().await?;

		serde_json::from_str(&body).map_err(|err| {
            let error_details = serde_json::from_str::<Value>(&body).ok().map(|details| {
                let error = details["error"].as_str();

                let reason = details["reason"].as_str();

                format!(
                    "{}: {}",
                    error.unwrap_or_default(),
                    reason.unwrap_or_default()
                )
            });

            if error_details.is_none() {
                debug!("Failed request response body: {:?}", body);
            }

            anyhow!(
                "Could not parse response JSON, URL is {}, status is {}: {:?}: {}",
                request_url,
                status,
                err,
                error_details.unwrap_or_default()
            )
        })
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
			.sign_origin(
				&StacksPrivateKey::from_slice(
					&self
						.config
						.stacks_credentials
						.private_key()
						.secret_bytes(),
				)
				.unwrap(),
			)
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
					.get(self.cachebust(self.get_transation_details_url(txid)))
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
			.get(self.cachebust(self.nonce_url()))
			.send()
			.await?
			.json()
			.await?)
	}

	/// Get the block height of the contract
	pub async fn get_contract_block_height(
		&mut self,
		name: ContractName,
	) -> anyhow::Result<u32> {
		let addr = self.config.stacks_credentials.address();
		let id = QualifiedContractIdentifier::new(
			StandardPrincipalData(
				addr.version() as u8,
				addr.hash().as_ref().try_into().unwrap(),
			),
			name,
		);

		let res: Value = self
			.http_client
			.get(self.contract_info_url(id.to_string()))
			.send()
			.await?
			.json()
			.await?;

		Ok(res["block_height"].as_u64().unwrap() as u32)
	}

	async fn calculate_fee(&self, tx_len: u64) -> anyhow::Result<u64> {
		let fee_rate: u64 = self
			.http_client
			.get(self.fee_url())
			.send()
			.await?
			.json()
			.await?;

		// TODO: Figure out what's the right multiplier #98
		Ok(fee_rate * tx_len * 100)
	}

	fn transaction_url(&self) -> reqwest::Url {
		self.config
			.stacks_node_url
			.join("/v2/transactions")
			.unwrap()
	}

	fn contract_info_url(&self, id: impl AsRef<str>) -> reqwest::Url {
		self.config
			.stacks_node_url
			.join(&format!("/extended/v1/contract/{}", id.as_ref()))
			.unwrap()
	}

	fn get_transation_details_url(&self, txid: StacksTxId) -> reqwest::Url {
		self.config
			.stacks_node_url
			.join(&format!("/extended/v1/tx/{}", txid))
			.unwrap()
	}

	fn cachebust(&self, mut url: reqwest::Url) -> reqwest::Url {
		let mut rng = thread_rng();
		let random_string: String =
			(0..16).map(|_| rng.sample(Alphanumeric) as char).collect();

		let mut query = url
			.query()
			.map(|query| query.to_string())
			.unwrap_or_default();

		query = match query.is_empty() {
			true => format!("cachebuster={}", random_string),
			false => format!("{}&cachebuster={}", query, random_string),
		};

		url.set_query(Some(&query));

		url
	}

	fn nonce_url(&self) -> reqwest::Url {
		let path = format!(
			"/extended/v1/address/{}/nonces",
			self.config.stacks_credentials.address(),
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
	use super::*;
	use crate::config::Config;

	// These integration tests are for exploration/experimentation but should be
	// removed once we have more decent tests
	#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
	#[ignore]
	async fn get_nonce_info() {
		let config = Config::from_path("./testing/config.json")
			.expect("Failed to find config file");
		let http_client = reqwest::Client::new();

		let mut stacks_client = StacksClient::new(config, http_client);

		let nonce_info = stacks_client.get_nonce_info().await.unwrap();
		assert_eq!(nonce_info.possible_next_nonce, 122);

		assert!(true);
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
	#[ignore]
	async fn get_fee_rate() {
		let config = Config::from_path("./testing/config.json")
			.expect("Failed to find config file");
		let http_client = reqwest::Client::new();

		let stacks_client = StacksClient::new(config, http_client);

		stacks_client.calculate_fee(123).await.unwrap();
	}
}

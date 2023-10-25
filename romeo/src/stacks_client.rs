//! Stacks client

use std::{io::Cursor, time::Duration};

use anyhow::{anyhow, Error};
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
use futures::{stream::FuturesUnordered, StreamExt};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest::{RequestBuilder, Response};
use serde::de::DeserializeOwned;
use serde_json::Value;
use stacks_core::{codec::Codec, uint::Uint256, wallet::Credentials};
use tokio::time::sleep;
use tracing::{trace, warn};
use url::Url;

use crate::event::TransactionStatus;

const BLOCK_POLLING_INTERVAL: Duration = Duration::from_secs(5);

/// Stateful client for creating and broadcasting Stacks transactions
///
/// This client keeps track of the last executed nonce for the given
/// key.
#[derive(Debug, Clone)]
pub struct StacksClient {
	hiro_api_key: Option<String>,
	stacks_node_url: Url,
	stacks_credentials: Credentials,
	http_client: reqwest::Client,
}

impl StacksClient {
	/// Create a new StacksClient
	pub fn new(
		hiro_api_key: Option<String>,
		stacks_node_url: Url,
		stacks_credentials: Credentials,
		http_client: reqwest::Client,
	) -> Self {
		Self {
			hiro_api_key,
			stacks_node_url,
			stacks_credentials,
			http_client,
		}
	}

	async fn send_request<T>(
		&self,
		builder: RequestBuilder,
	) -> anyhow::Result<T>
	where
		T: DeserializeOwned,
	{
		let res = self.retry(self.add_stacks_api_key(builder)).await?;
		res.error_for_status_ref().expect("retry propagates errors");

		let body = res.text().await?;

		serde_json::from_str(&body).map_err(|e| anyhow!("{e}: body {body}"))
	}

	/// if hiro_api_key is set, add it to the request
	fn add_stacks_api_key(&self, request: RequestBuilder) -> RequestBuilder {
		match &self.hiro_api_key {
			Some(api_key) => request.header("x-hiro-api-key", api_key),
			None => request,
		}
	}

	/// Sign and broadcast an unsigned stacks transaction
	pub async fn sign_and_broadcast(
		&self,
		mut tx: StacksTransaction,
	) -> anyhow::Result<StacksTxId> {
		#[cfg(debug_assertions)]
		{
			sleep(Duration::from_secs(3)).await;
		}

		tx.set_origin_nonce(self.get_nonce_info().await?.possible_next_nonce);
		tx.set_tx_fee(self.calculate_fee(tx.tx_len()).await?);

		tx.anchor_mode = TransactionAnchorMode::Any;
		tx.post_condition_mode = TransactionPostConditionMode::Allow;
		tx.chain_id = CHAIN_ID_TESTNET;

		let mut signer = StacksTransactionSigner::new(&tx);

		signer
			.sign_origin(
				&StacksPrivateKey::from_slice(
					&self.stacks_credentials.private_key().secret_bytes(),
				)
				.unwrap(),
			)
			.unwrap();

		tx = signer.get_tx().unwrap();

		let mut tx_bytes = vec![];
		tx.consensus_serialize(&mut tx_bytes).unwrap();

		let res = self
			.send_request({
				let tx_bytes = tx_bytes.clone();

				self.http_client
					.post(self.transaction_url())
					.header("Content-type", "application/octet-stream")
					.body(tx_bytes)
			})
			.await?;

		Ok(res)
	}

	/// Get transaction status for a given txid
	pub async fn get_transation_status(
		&self,
		txid: StacksTxId,
	) -> anyhow::Result<TransactionStatus> {
		let res: anyhow::Result<Value> = self
			.send_request(
				self.http_client
					.get(self.cachebust(self.get_transation_details_url(txid)))
					.header("Accept", "application/json"),
			)
			.await;

		let tx_status_str = match res {
			Ok(json) => json["tx_status"]
				.as_str()
				.expect("Could not get raw transaction from response")
				.to_string(),
			// Stacks node sometimes returns 404 for pending transactions
			// :shrug:
			Err(err) if err.to_string().contains("404 Not Found") => {
				"pending".to_string()
			}
			err => panic!("Unknown transation status: {:?}", err),
		};

		Ok(match tx_status_str.as_str() {
			"pending" => TransactionStatus::Broadcasted,
			"success" => TransactionStatus::Confirmed,
			"abort_by_response" => TransactionStatus::Rejected,
			status => panic!("Unknown transation status: {}", status),
		})
	}

	async fn get_nonce_info(&self) -> anyhow::Result<NonceInfo> {
		self.send_request(
			self.http_client.get(self.cachebust(self.nonce_url())),
		)
		.await
	}

	/// Get the block height of the contract
	pub async fn get_contract_block_height(
		&self,
		name: ContractName,
	) -> anyhow::Result<u32> {
		let addr = self.stacks_credentials.address();
		let id = QualifiedContractIdentifier::new(
			StandardPrincipalData(
				addr.version() as u8,
				addr.hash().as_ref().try_into().unwrap(),
			),
			name,
		);

		let req_builder =
			self.http_client.get(self.contract_info_url(id.to_string()));

		self.send_error_guarded_request(req_builder, "block_height")
			.await
	}

	/// Get the Bitcoin block height for a Stacks block height
	pub async fn get_bitcoin_block_height(
		&self,
		block_height: u32,
	) -> anyhow::Result<u32> {
		self.send_error_guarded_request::<u32>(
			self.http_client.get(self.block_by_height_url(block_height)),
			"burn_block_height",
		)
		.await
	}

	/// Get the block at height
	pub async fn get_block(
		&self,
		block_height: u32,
	) -> anyhow::Result<Vec<StacksTransaction>> {
		let raw_txids: Value = loop {
			let maybe_response: Result<Value, Error> = self
				.send_error_guarded_request(
					self.http_client
						.get(self.block_by_height_url(block_height)),
					"txs",
				)
				.await;

			if let Ok(txs_value) = maybe_response {
				if txs_value.is_array() {
					trace!("Found Stacks block of height {}", block_height);
					break txs_value;
				}
			}

			trace!("Stacks block not found, retrying...");
			sleep(BLOCK_POLLING_INTERVAL).await;
		};

		raw_txids
			.as_array()
			.expect("An array, found {raw_txids:?")
			.iter()
			.map(|id| {
				StacksTxId::from_hex(
					id.as_str().unwrap().trim_start_matches("0x"),
				)
				.unwrap()
			})
			.map(|txid| self.get_transaction(txid))
			.collect::<FuturesUnordered<_>>()
			.collect::<Vec<_>>()
			.await
			.into_iter()
			.collect::<Result<Vec<StacksTransaction>, _>>()
	}

	/// Get the block at height
	pub async fn get_transaction(
		&self,
		id: StacksTxId,
	) -> anyhow::Result<StacksTransaction> {
		let res: Value = self
			.send_error_guarded_request(
				self.http_client
					.get(self.get_raw_transaction_url(id))
					.header("Accept", "application/octet-stream"),
				"raw_tx",
			)
			.await?;

		let raw_tx = res.as_str().unwrap().trim_start_matches("0x");
		let bytes = hex::decode(raw_tx).unwrap();
		let tx =
			StacksTransaction::consensus_deserialize(&mut &bytes[..]).unwrap();

		Ok(tx)
	}

	/// Get the block hash for a given Bitcoin height
	pub async fn get_block_hash_from_bitcoin_height(
		&self,
		height: u32,
	) -> anyhow::Result<Uint256> {
		let res: Value = self
			.send_error_guarded_request(
				self.http_client
					.get(self.block_by_bitcoin_height_url(height))
					.header("Accept", "application/json"),
				"hash",
			)
			.await?;

		let hash_str = res
			.as_str()
			.expect("Could not get block hash: {res:?}")
			.trim_start_matches("0x");
		let hash_bytes = hex::decode(hash_str)?;

		Ok(Uint256::deserialize(&mut Cursor::new(hash_bytes))?)
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
		self.stacks_node_url.join("/v2/transactions").unwrap()
	}

	fn get_raw_transaction_url(&self, txid: StacksTxId) -> reqwest::Url {
		self.stacks_node_url
			.join(&format!("/extended/v1/tx/{}/raw", txid))
			.unwrap()
	}

	fn block_by_height_url(&self, height: u32) -> reqwest::Url {
		self.stacks_node_url
			.join(&format!("/extended/v1/block/by_height/{}", height))
			.unwrap()
	}

	fn block_by_bitcoin_height_url(&self, height: u32) -> reqwest::Url {
		self.stacks_node_url
			.join(&format!(
				"/extended/v1/block/by_burn_block_height/{}",
				height
			))
			.unwrap()
	}

	fn contract_info_url(&self, id: impl AsRef<str>) -> reqwest::Url {
		self.stacks_node_url
			.join(&format!("/extended/v1/contract/{}", id.as_ref()))
			.unwrap()
	}

	fn get_transation_details_url(&self, txid: StacksTxId) -> reqwest::Url {
		self.stacks_node_url
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
			self.stacks_credentials.address(),
		);

		self.stacks_node_url.join(&path).unwrap()
	}

	fn fee_url(&self) -> reqwest::Url {
		self.stacks_node_url.join("/v2/fees/transfer").unwrap()
	}

	async fn send_error_guarded_request<T>(
		&self,
		req: RequestBuilder,
		index: &str,
	) -> anyhow::Result<T>
	where
		T: DeserializeOwned,
	{
		let res: Value = self.send_request(req).await?;

		if let Some(err) = res["error"].as_str() {
			let reason = res["reason"].as_str();
			Err(anyhow!("{err}; reason: {reason:?}"))
		} else {
			Ok(serde_json::from_value(res[index].clone())?)
		}
	}

	async fn retry(&self, builder: RequestBuilder) -> anyhow::Result<Response> {
		use backoff::Error as BackOffError;

		let operation = || async {
			let request = builder
				.try_clone()
				.expect("not a stream")
				.build()
				.map_err(|e| BackOffError::permanent(anyhow!(e)))?;

			self.http_client
				.execute(request)
				.await
				.and_then(Response::error_for_status)
				.map_err(|e| {
					if e.is_request() {
						BackOffError::transient(anyhow!(e))
					} else if e.is_status() {
						match e
							.status()
							.expect("Is status <-> has status: qed")
							.as_u16()
						{
							429 | 522 => BackOffError::transient(anyhow!(e)),
							_ => BackOffError::permanent(anyhow!(e)),
						}
					} else {
						BackOffError::permanent(anyhow!(e))
					}
				})
		};

		let notify = |err, duration| {
			warn!("Retrying in {:?} after error: {:?}", duration, err);
		};

		backoff::future::retry_notify(
			backoff::ExponentialBackoff::default(),
			operation,
			notify,
		)
		.await
	}
}

#[derive(serde::Deserialize)]
struct NonceInfo {
	possible_next_nonce: u64,
}

#[cfg(test)]
mod tests {

	use std::ops::Add;

	use assert_matches::assert_matches;

	use super::*;
	use crate::config::Config;

	// These integration tests are for exploration/experimentation but should be
	// removed once we have more decent tests
	#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
	#[ignore]
	async fn get_nonce_info() {
		let Config {
			hiro_api_key,
			stacks_node_url,
			stacks_credentials,
			..
		} = Config::from_path("./testing/config.json")
			.expect("Failed to find config file");
		let http_client = reqwest::Client::new();

		let stacks_client = StacksClient::new(
			hiro_api_key,
			stacks_node_url,
			stacks_credentials,
			http_client,
		);

		let nonce_info = stacks_client.get_nonce_info().await.unwrap();
		assert_eq!(nonce_info.possible_next_nonce, 122);
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
	#[ignore]
	async fn get_fee_rate() {
		let Config {
			hiro_api_key,
			stacks_node_url,
			stacks_credentials,
			..
		} = Config::from_path("./testing/config.json")
			.expect("Failed to find config file");
		let http_client = reqwest::Client::new();

		let stacks_client = StacksClient::new(
			hiro_api_key,
			stacks_node_url,
			stacks_credentials,
			http_client,
		);

		stacks_client.calculate_fee(123).await.unwrap();
	}

	#[tokio::test]
	async fn missing_contract_errors() {
		let Config {
			hiro_api_key,
			stacks_credentials,
			..
		} = Config::from_path("../devenv/sbtc/docker/config.json")
			.expect("Failed to find config file");

		let contract = "missingcontract";

		let mut server = mockito::Server::new();
		let m = server
			.mock(
				"GET",
				format!(
					"/extended/v1/contract/{}.{contract}",
					stacks_credentials.address()
				)
				.as_str(),
			)
			.with_status(404)
			.create();

		let stacks_client = StacksClient::new(
			hiro_api_key,
			server.url().parse().unwrap(),
			stacks_credentials,
			reqwest::Client::new(),
		);

		assert_eq!(
			stacks_client
				.get_contract_block_height(contract.try_into().unwrap(),)
				.await
				.expect_err("contract not deployed")
				.downcast::<reqwest::Error>()
				.unwrap()
				.status()
				.unwrap(),
			404
		);

		m.assert();
	}

	#[tokio::test]
	async fn send_request_prints_body_on_err() {
		let Config {
			stacks_credentials, ..
		} = Config::from_path("../devenv/sbtc/docker/config.json")
			.expect("Failed to find config file");

		let contract = "missingcontract";

		let mut server = mockito::Server::new();
		let path = format!(
			"/extended/v1/contract/{}.{contract}",
			stacks_credentials.address()
		);
		let body = r#"{ "block_height": 2 }"#;
		let m = server.mock("GET", path.as_str()).with_body(body).create();

		let stacks_client = StacksClient::new(
			None,
			server.url().parse().unwrap(),
			stacks_credentials,
			reqwest::Client::new(),
		);

		let req_builder =
			stacks_client.http_client.get(server.url().add(&path));

		assert_matches!(stacks_client.send_request::<u32>(req_builder).await, Err(e)=>{
			assert!(e.to_string().contains(body));
		});

		m.assert();
	}

	#[tokio::test]
	async fn get_contract_block_height_positive() {
		let Config {
			stacks_credentials, ..
		} = Config::from_path("../devenv/sbtc/docker/config.json")
			.expect("Failed to find config file");

		let contract = "contractname";

		let mut server = mockito::Server::new();
		let path = format!(
			"/extended/v1/contract/{}.{contract}",
			stacks_credentials.address()
		);
		let m = server
			.mock("GET", path.as_str())
			.with_body(r#"{ "block_height": 2 }"#)
			.create();

		let stacks_client = StacksClient::new(
			None,
			server.url().parse().unwrap(),
			stacks_credentials,
			reqwest::Client::new(),
		);

		assert_eq!(
			stacks_client
				.get_contract_block_height(contract.try_into().unwrap(),)
				.await
				.unwrap(),
			2
		);

		m.assert();
	}

	#[tokio::test]
	async fn send_error_guarded_request_errors_if_error_field_present() {
		let Config {
			stacks_credentials, ..
		} = Config::from_path("../devenv/sbtc/docker/config.json")
			.expect("Failed to find config file");

		let mut server = mockito::Server::new();
		let path = format!(
			"/extended/v1/contract/{}.contract",
			stacks_credentials.address()
		);
		let m = server
			.mock("GET", path.as_str())
			.with_body(r#"{ "any": 1, "error": "Oops", "reason": "Ay!" }"#)
			.create();

		let stacks_client = StacksClient::new(
			None,
			server.url().parse().unwrap(),
			stacks_credentials,
			reqwest::Client::new(),
		);

		let req_builder =
			stacks_client.http_client.get(server.url().add(&path));

		let error = stacks_client
			.send_error_guarded_request::<()>(req_builder, "any")
			.await
			.expect_err("response body contains an error field");
		assert!(error.to_string().contains("reason"));

		m.assert();
	}
}

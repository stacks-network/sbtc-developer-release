//! Stacks client

use std::{io::Cursor, sync::Arc, time::Duration};

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
use futures::Future;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest::{Request, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Value;
use stacks_core::{codec::Codec, uint::Uint256};
use tokio::{
	sync::{Mutex, MutexGuard},
	time::sleep,
};
use tracing::{debug, trace, warn};

use crate::{config::Config, event::TransactionStatus};

const BLOCK_POLLING_INTERVAL: Duration = Duration::from_secs(5);

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

	async fn send_request<B, T>(&self, request_builder: B) -> anyhow::Result<T>
	where
		B: Clone + Fn() -> Request,
		T: DeserializeOwned,
	{
		let request_url = request_builder().url().to_string();

		let res = retry(|| {
			self.http_client
				.execute(self.add_stacks_api_key(request_builder()))
		})
		.await?;

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

	/// if hiro_api_key is set, add it to the request
	fn add_stacks_api_key(&self, request: Request) -> Request {
		match &self.config.hiro_api_key {
			Some(api_key) => {
				RequestBuilder::from_parts(self.http_client.clone(), request)
					.header("x-hiro-api-key", api_key)
					.build()
					.unwrap()
			}
			None => request,
		}
	}

	/// Sign and broadcast an unsigned stacks transaction
	pub async fn sign_and_broadcast(
		&mut self,
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
			.send_request(|| {
				let tx_bytes = tx_bytes.clone();

				self.http_client
					.post(self.transaction_url())
					.header("Content-type", "application/octet-stream")
					.body(tx_bytes)
					.build()
					.unwrap()
			})
			.await?;

		Ok(res)
	}

	/// Get transaction status for a given txid
	pub async fn get_transation_status(
		&mut self,
		txid: StacksTxId,
	) -> anyhow::Result<TransactionStatus> {
		let res: anyhow::Result<Value> = self
			.send_request(|| {
				self.http_client
					.get(self.cachebust(self.get_transation_details_url(txid)))
					.header("Accept", "application/json")
					.build()
					.unwrap()
			})
			.await;

		let tx_status_str = match res {
			Ok(json) => json["tx_status"]
				.as_str()
				.map(|s| s.to_string())
				.expect("Could not get raw transaction from response"),
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

	async fn get_nonce_info(&mut self) -> anyhow::Result<NonceInfo> {
		self.send_request(|| {
			self.http_client
				.get(self.cachebust(self.nonce_url()))
				.build()
				.unwrap()
		})
		.await
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
			.send_request(|| {
				self.http_client
					.get(self.contract_info_url(id.to_string()))
					.build()
					.unwrap()
			})
			.await?;

		if let Some(err) = res["error"].as_str() {
			Err(Error::msg(err.to_string()))
		} else {
			Ok(res["block_height"].as_u64().unwrap() as u32)
		}
	}

	/// Get the Bitcoin block height for a Stacks block height
	pub async fn get_bitcoin_block_height(
		&mut self,
		block_height: u32,
	) -> anyhow::Result<u32> {
		let res: Value = self
			.send_request(|| {
				self.http_client
					.get(self.block_by_height_url(block_height))
					.build()
					.unwrap()
			})
			.await?;

		Ok(res["burn_block_height"].as_u64().unwrap() as u32)
	}

	/// Get the block at height
	pub async fn get_block(
		&mut self,
		block_height: u32,
	) -> anyhow::Result<Vec<StacksTransaction>> {
		let res: Value = loop {
			let maybe_response: Result<Value, Error> = self
				.send_request(|| {
					self.http_client
						.get(self.block_by_height_url(block_height))
						.build()
						.unwrap()
				})
				.await;

			if let Ok(inner_response) = maybe_response {
				if inner_response["txs"].is_array() {
					trace!("Found Stacks block of height {}", block_height);
					break inner_response;
				}
			}

			trace!("Stacks block not found, retrying...");
			sleep(BLOCK_POLLING_INTERVAL).await;
		};

		let tx_ids: Vec<StacksTxId> = res["txs"]
			.as_array()
			.unwrap_or_else(|| {
				panic!("Could not get txs from response: {:?}", res)
			})
			.iter()
			.map(|id| {
				let mut id = id.as_str().unwrap().to_string();
				id = id.replace("0x", "");

				StacksTxId::from_hex(&id).unwrap()
			})
			.collect();

		let mut txs = Vec::with_capacity(tx_ids.len());

		for id in tx_ids {
			let tx = self.get_transaction(id).await?;
			txs.push(tx);
		}

		Ok(txs)
	}

	/// Get the block at height
	pub async fn get_transaction(
		&mut self,
		id: StacksTxId,
	) -> anyhow::Result<StacksTransaction> {
		let res: Value = self
			.send_request(|| {
				self.http_client
					.get(self.get_raw_transaction_url(id))
					.header("Accept", "application/octet-stream")
					.build()
					.unwrap()
			})
			.await?;

		let mut raw_tx: String = res["raw_tx"].as_str().unwrap().to_string();
		raw_tx = raw_tx.replace("0x", "");

		let bytes = hex::decode(raw_tx).unwrap();
		let tx =
			StacksTransaction::consensus_deserialize(&mut &bytes[..]).unwrap();

		Ok(tx)
	}

	/// Get the block hash for a given Bitcoin height
	pub async fn get_block_hash_from_bitcoin_height(
		&mut self,
		height: u32,
	) -> anyhow::Result<Uint256> {
		let res: Value = self
			.send_request(|| {
				self.http_client
					.get(self.block_by_bitcoin_height_url(height))
					.header("Accept", "application/json")
					.build()
					.unwrap()
			})
			.await?;

		let hash_str = res["hash"]
			.as_str()
			.unwrap_or_else(|| panic!("Could not get block hash: {:?}", res));
		let hash_bytes = hex::decode(hash_str.replace("0x", ""))?;

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
		self.config
			.stacks_node_url
			.join("/v2/transactions")
			.unwrap()
	}

	fn get_raw_transaction_url(&self, txid: StacksTxId) -> reqwest::Url {
		self.config
			.stacks_node_url
			.join(&format!("/extended/v1/tx/{}/raw", txid))
			.unwrap()
	}

	fn block_by_height_url(&self, height: u32) -> reqwest::Url {
		self.config
			.stacks_node_url
			.join(&format!("/extended/v1/block/by_height/{}", height))
			.unwrap()
	}

	fn block_by_bitcoin_height_url(&self, height: u32) -> reqwest::Url {
		self.config
			.stacks_node_url
			.join(&format!(
				"/extended/v1/block/by_burn_block_height/{}",
				height
			))
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

async fn retry<O, Fut>(operation: O) -> anyhow::Result<Response>
where
	O: Clone + Fn() -> Fut,
	Fut: Future<Output = Result<Response, reqwest::Error>>,
{
	let operation = || async {
		operation.clone()()
			.await
			.and_then(Response::error_for_status)
			.map_err(|err| {
				if err.is_request() {
					backoff::Error::transient(anyhow::anyhow!(err))
				} else if err.is_status() {
					// Impossible not to have a status code at this section. May
					// as well be a teapot.
					let status_code_number = err
						.status()
						.unwrap_or(StatusCode::IM_A_TEAPOT)
						.as_u16();
					match status_code_number {
						429 | 522 => {
							backoff::Error::transient(anyhow::anyhow!(err))
						}
						_ => backoff::Error::permanent(anyhow::anyhow!(err)),
					}
				} else {
					backoff::Error::permanent(anyhow::anyhow!(err))
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

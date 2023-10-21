//! System

use std::{fs::create_dir_all, io::Cursor};

use bdk::bitcoin::Txid as BitcoinTxId;
use blockstack_lib::{
	burnchains::Txid as StacksTxId,
	chainstate::stacks::{
		StacksTransaction, TransactionAuth, TransactionContractCall,
		TransactionPayload, TransactionSpendingCondition, TransactionVersion,
	},
	codec::StacksMessageCodec,
	types::chainstate::{StacksAddress, StacksPublicKey},
	vm::{types::Value, ClarityName},
};
use sbtc_core::operations::op_return::withdrawal_fulfillment::create_outputs;
use stacks_core::{codec::Codec, BlockId, Network as StacksNetwork};
use tokio::{
	fs::{File, OpenOptions},
	io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
	sync::mpsc,
	task::JoinHandle,
};
use tracing::{info, trace};

use crate::{
	bitcoin_client::Client as BitcoinClient,
	config::Config,
	event::Event,
	proof_data::{ProofData, ProofDataClarityValues},
	stacks_client::{LockedClient, StacksClient},
	state,
	state::{DepositInfo, WithdrawalInfo},
	task::Task,
};

/// The main run loop of this system.
/// This function feeds all events to the `state::update` function and spawns
/// all tasks returned from this function.
///
/// The system is bootstrapped by emitting the CreateAssetContract task.
pub async fn run(config: Config) {
	let (tx, mut rx) = mpsc::channel::<Event>(128); // TODO: Make capacity configurable
	let bitcoin_client = BitcoinClient::new(config.clone())
		.expect("Failed to instantiate bitcoin client");
	let stacks_client: LockedClient =
		StacksClient::new(config.clone(), reqwest::Client::new()).into();

	info!("Starting replay of persisted events");

	let (mut storage, mut state) =
		Storage::load_and_replay(&config, state::State::new()).await;

	info!("Replay finished with state: {:?}", state);

	let bootstrap_tasks = state.bootstrap();

	// Bootstrap
	for task in bootstrap_tasks {
		spawn(
			config.clone(),
			bitcoin_client.clone(),
			stacks_client.clone(),
			task,
			tx.clone(),
		);
	}

	while let Some(event) = rx.recv().await {
		storage.record(&event).await;

		let tasks = state.update(event, &config);
		trace!("State: {}", serde_json::to_string(&state).unwrap());

		for task in tasks {
			spawn(
				config.clone(),
				bitcoin_client.clone(),
				stacks_client.clone(),
				task,
				tx.clone(),
			);
		}
	}
}

struct Storage(BufWriter<File>);

impl Storage {
	async fn load_and_replay(
		config: &Config,
		mut state: state::State,
	) -> (Self, state::State) {
		create_dir_all(&config.state_directory).unwrap();

		let mut file = OpenOptions::new()
			.create(true)
			.read(true)
			.write(true)
			.append(true)
			.open(config.state_directory.join("log.ndjson"))
			.await
			.unwrap();

		let mut r = BufReader::new(&mut file).lines();

		while let Some(line) = r.next_line().await.unwrap() {
			let event: Event = serde_json::from_str(&line).unwrap();

			state.update(event, config);
		}

		(Self(BufWriter::new(file)), state)
	}

	async fn record(&mut self, event: &Event) {
		let bytes = serde_json::to_vec(event).unwrap();
		self.0.write_all(&bytes).await.unwrap();
		self.0.write_all(b"\n").await.unwrap();
		self.0.flush().await.unwrap();
	}
}

#[tracing::instrument(skip(config, bitcoin_client, stacks_client, result))]
fn spawn(
	config: Config,
	bitcoin_client: BitcoinClient,
	stacks_client: LockedClient,
	task: Task,
	result: mpsc::Sender<Event>,
) -> JoinHandle<()> {
	info!("Spawning");

	tokio::task::spawn(async move {
		let event =
			run_task(&config, bitcoin_client, stacks_client, task).await;
		result.send(event).await.expect("Failed to return event");
	})
}

async fn run_task(
	config: &Config,
	bitcoin_client: BitcoinClient,
	stacks_client: LockedClient,
	task: Task,
) -> Event {
	match task {
		Task::GetContractBlockHeight => {
			get_contract_block_height(config, stacks_client).await
		}
		Task::UpdateContractPublicKey => {
			update_contract_public_key(config, stacks_client).await
		}
		Task::CreateMint(deposit_info) => {
			mint_asset(config, bitcoin_client, stacks_client, deposit_info)
				.await
		}
		Task::CreateBurn(withdrawal_info) => {
			burn_asset(config, bitcoin_client, stacks_client, withdrawal_info)
				.await
		}
		Task::CreateFulfillment(fulfillment_info) => {
			fulfill_asset(
				config,
				bitcoin_client,
				stacks_client,
				fulfillment_info,
			)
			.await
		}
		Task::CheckBitcoinTransactionStatus(txid) => {
			check_bitcoin_transaction_status(config, bitcoin_client, txid).await
		}
		Task::CheckStacksTransactionStatus(txid) => {
			check_stacks_transaction_status(stacks_client, txid).await
		}
		Task::FetchStacksBlock(block_height) => {
			fetch_stacks_block(stacks_client, block_height).await
		}
		Task::FetchBitcoinBlock(block_height) => {
			fetch_bitcoin_block(bitcoin_client, block_height).await
		}
	}
}

async fn get_contract_block_height(
	config: &Config,
	client: LockedClient,
) -> Event {
	let block_height = client
		.lock()
		.await
		.get_contract_block_height(config.contract_name.clone())
		.await
		.expect("Could not get block height. Binary needs to be restarted after contract deployment.");

	let bitcoin_block_height = client
		.lock()
		.await
		.get_bitcoin_block_height(block_height)
		.await
		.expect("Could not get burnchain block height. Binary needs to be restarted after bitcoin node is online again.");

	Event::ContractBlockHeight(block_height, bitcoin_block_height)
}

async fn update_contract_public_key(
	config: &Config,
	stacks_client: LockedClient,
) -> Event {
	let public_key = StacksPublicKey::from_slice(
		&config.stacks_credentials.public_key().serialize(),
	)
	.unwrap();

	let tx_auth = TransactionAuth::Standard(
		TransactionSpendingCondition::new_singlesig_p2pkh(public_key).unwrap(),
	);

	let function_args = vec![Value::buff_from(
		config
			.bitcoin_credentials
			.public_key_p2tr()
			.serialize()
			.try_into()
			.unwrap(),
	)
	.expect("Cannot convert public key into a Clarity Value")];

	let addr = StacksAddress::consensus_deserialize(&mut Cursor::new(
		config.stacks_credentials.address().serialize_to_vec(),
	))
	.unwrap();

	let tx_payload =
		TransactionPayload::ContractCall(TransactionContractCall {
			address: addr,
			contract_name: config.contract_name.clone(),
			function_name: ClarityName::from("set-bitcoin-wallet-public-key"),
			function_args,
		});

	let tx_version = match config.stacks_network {
		StacksNetwork::Mainnet => TransactionVersion::Mainnet,
		StacksNetwork::Testnet => TransactionVersion::Testnet,
	};

	let tx = StacksTransaction::new(tx_version, tx_auth, tx_payload);

	let txid = stacks_client
		.lock()
		.await
		.sign_and_broadcast(tx)
		.await
		.expect("Unable to sign and broadcast the mint transaction");

	Event::ContractPublicKeySetBroadcasted(txid)
}

async fn mint_asset(
	config: &Config,
	bitcoin_client: BitcoinClient,
	stacks_client: LockedClient,
	deposit_info: DepositInfo,
) -> Event {
	let proof_data = get_tx_proof(
		&bitcoin_client,
		deposit_info.block_height,
		deposit_info.txid,
	)
	.await;

	let public_key = StacksPublicKey::from_slice(
		&config.stacks_credentials.public_key().serialize(),
	)
	.unwrap();

	let tx_auth = TransactionAuth::Standard(
		TransactionSpendingCondition::new_singlesig_p2pkh(public_key).unwrap(),
	);

	let function_args = vec![
		Value::UInt(deposit_info.amount as u128),
		Value::from(deposit_info.recipient.clone()),
		proof_data.txid,
		proof_data.block_height,
		proof_data.merkle_path,
		proof_data.tx_index,
		proof_data.block_header,
	];

	let addr = StacksAddress::consensus_deserialize(&mut Cursor::new(
		config.stacks_credentials.address().serialize_to_vec(),
	))
	.unwrap();

	let tx_payload =
		TransactionPayload::ContractCall(TransactionContractCall {
			address: addr,
			contract_name: config.contract_name.clone(),
			function_name: ClarityName::from("mint"),
			function_args,
		});

	let tx_version = match config.stacks_network {
		StacksNetwork::Mainnet => TransactionVersion::Mainnet,
		StacksNetwork::Testnet => TransactionVersion::Testnet,
	};

	let tx = StacksTransaction::new(tx_version, tx_auth, tx_payload);

	let txid = stacks_client
		.lock()
		.await
		.sign_and_broadcast(tx)
		.await
		.expect("Unable to sign and broadcast the mint transaction");

	Event::MintBroadcasted(deposit_info, txid)
}

async fn burn_asset(
	config: &Config,
	bitcoin_client: BitcoinClient,
	stacks_client: LockedClient,
	withdrawal_info: WithdrawalInfo,
) -> Event {
	let proof_data = get_tx_proof(
		&bitcoin_client,
		withdrawal_info.block_height,
		withdrawal_info.txid,
	)
	.await;

	let public_key = StacksPublicKey::from_slice(
		&config.stacks_credentials.public_key().serialize(),
	)
	.unwrap();

	let tx_auth = TransactionAuth::Standard(
		TransactionSpendingCondition::new_singlesig_p2pkh(public_key).unwrap(),
	);

	let function_args = vec![
		Value::UInt(withdrawal_info.amount as u128),
		Value::from(withdrawal_info.source.clone()),
		proof_data.txid,
		proof_data.block_height,
		proof_data.merkle_path,
		proof_data.tx_index,
		proof_data.block_header,
	];

	let addr = StacksAddress::consensus_deserialize(&mut Cursor::new(
		config.stacks_credentials.address().serialize_to_vec(),
	))
	.unwrap();

	let tx_payload =
		TransactionPayload::ContractCall(TransactionContractCall {
			address: addr,
			contract_name: config.contract_name.clone(),
			function_name: ClarityName::from("burn"),
			function_args,
		});

	let tx_version = match config.stacks_network {
		StacksNetwork::Mainnet => TransactionVersion::Mainnet,
		StacksNetwork::Testnet => TransactionVersion::Testnet,
	};

	let tx = StacksTransaction::new(tx_version, tx_auth, tx_payload);

	let txid = stacks_client
		.lock()
		.await
		.sign_and_broadcast(tx)
		.await
		.expect("Unable to sign and broadcast the mint transaction");

	Event::BurnBroadcasted(withdrawal_info, txid)
}

async fn fulfill_asset(
	config: &Config,
	bitcoin_client: BitcoinClient,
	stacks_client: LockedClient,
	withdrawal_info: WithdrawalInfo,
) -> Event {
	let stacks_chain_tip = stacks_client
		.lock()
		.await
		.get_block_hash_from_bitcoin_height(withdrawal_info.block_height)
		.await
		.expect("Unable to get stacks block hash");

	let outputs = create_outputs(
		BlockId::new(stacks_chain_tip),
		config.bitcoin_network,
		&withdrawal_info.recipient,
		withdrawal_info.amount,
	)
	.expect("Could not create withdrawal fulfillment outputs");

	let txid = bitcoin_client
		.sign_and_broadcast(outputs.to_vec())
		.await
		.expect(
		"Unable to sign and broadcast the withdrawal fulfillment transaction",
	);

	Event::FulfillBroadcasted(withdrawal_info, txid)
}

async fn get_tx_proof(
	bitcoin_client: &BitcoinClient,
	height: u32,
	txid: BitcoinTxId,
) -> ProofDataClarityValues {
	let (_, block) = bitcoin_client
		.get_block(height)
		.await
		.expect("Failed to fetch block");

	let index = block
		.txdata
		.iter()
		.position(|tx| tx.txid() == txid)
		.expect("Failed to find transaction in block");

	ProofData::from_block_and_index(&block, index).to_values()
}

async fn check_bitcoin_transaction_status(
	_config: &Config,
	client: BitcoinClient,
	txid: BitcoinTxId,
) -> Event {
	let status = client
		.get_tx_status(txid)
		.await
		.expect("Could not get Bitcoin transaction status");

	Event::BitcoinTransactionUpdate(txid, status)
}

async fn check_stacks_transaction_status(
	client: LockedClient,
	txid: StacksTxId,
) -> Event {
	let status = client
		.lock()
		.await
		.get_transation_status(txid)
		.await
		.expect("Could not get Stacks transaction status");

	Event::StacksTransactionUpdate(txid, status)
}

async fn fetch_stacks_block(client: LockedClient, block_height: u32) -> Event {
	let txs = client
		.lock()
		.await
		.get_block(block_height)
		.await
		.expect("Failed to get Stacks block");

	Event::StacksBlock(block_height, txs)
}

async fn fetch_bitcoin_block(
	client: BitcoinClient,
	block_height: u32,
) -> Event {
	let (height, block) = client
		.get_block(block_height)
		.await
		.expect("Failed to fetch bitcoin block");

	Event::BitcoinBlock(height, block)
}

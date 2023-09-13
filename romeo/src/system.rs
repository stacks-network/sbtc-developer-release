//! System

use std::fs::create_dir_all;
use std::io::Cursor;

use bdk::bitcoin::Txid as BitcoinTxId;
use blockstack_lib::burnchains::Txid as StacksTxId;
use blockstack_lib::chainstate::stacks::TransactionContractCall;
use blockstack_lib::codec::StacksMessageCodec;
use blockstack_lib::types::chainstate::StacksAddress;
use blockstack_lib::types::chainstate::StacksPublicKey;

use blockstack_lib::vm::ClarityName;
use stacks_core::codec::Codec;
use tokio::fs::File;
use tokio::fs::OpenOptions;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::io::BufWriter;

use blockstack_lib::chainstate::stacks::StacksTransaction;
use blockstack_lib::chainstate::stacks::TransactionAuth;
use blockstack_lib::chainstate::stacks::TransactionPayload;
use blockstack_lib::chainstate::stacks::TransactionSmartContract;
use blockstack_lib::chainstate::stacks::TransactionSpendingCondition;
use blockstack_lib::chainstate::stacks::TransactionVersion;
use blockstack_lib::vm::types::Value;

use blockstack_lib::util_lib::strings::StacksString;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::debug;
use tracing::trace;

use crate::bitcoin_client::rpc::RPCClient as BitcoinRPCClient;
use crate::bitcoin_client::BitcoinClient;
use crate::config::Config;
use crate::event::Event;
use crate::proof_data::ProofData;
use crate::stacks_client::LockedClient;
use crate::stacks_client::StacksClient;
use crate::state;
use crate::state::DepositInfo;
use crate::task::Task;

/// The main run loop of this system.
/// This function feeds all events to the `state::update` function and spawns all tasks returned from this function.
///
/// The system is bootstrapped by emitting the CreateAssetContract task.
pub async fn run(config: Config) {
    let (tx, mut rx) = mpsc::channel::<Event>(128); // TODO: Make capacity configurable
    let bitcoin_client = BitcoinRPCClient::new(config.bitcoin_node_url.clone())
        .expect("Failed to instantiate bitcoin client");
    let stacks_client: LockedClient =
        StacksClient::new(config.clone(), reqwest::Client::new()).into();

    tracing::info!("Starting replay of persisted events");
    let (mut storage, state) = Storage::load_and_replay(&config, state::State::default()).await;
    tracing::info!("Replay finished with state: {:?}", state);

    let (mut state, bootstrap_task) = state::bootstrap(state);

    // Bootstrap
    spawn(
        config.clone(),
        bitcoin_client.clone(),
        stacks_client.clone(),
        bootstrap_task,
        tx.clone(),
    );

    while let Some(event) = rx.recv().await {
        storage.record(&event).await;

        let (next_state, tasks) = state::update(&config, state, event);

        trace!("State: {}", serde_json::to_string(&next_state).unwrap());

        for task in tasks {
            spawn(
                config.clone(),
                bitcoin_client.clone(),
                stacks_client.clone(),
                task,
                tx.clone(),
            );
        }

        state = next_state;
    }
}

struct Storage(BufWriter<File>);

impl Storage {
    async fn load_and_replay(config: &Config, mut state: state::State) -> (Self, state::State) {
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
            state = state::update(config, state, event).0;
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
    bitcoin_client: impl BitcoinClient + 'static,
    stacks_client: LockedClient,
    task: Task,
    result: mpsc::Sender<Event>,
) -> JoinHandle<()> {
    debug!("Spawning task");

    tokio::task::spawn(async move {
        let event = run_task(&config, bitcoin_client, stacks_client, task).await;
        result.send(event).await.expect("Failed to return event");
    })
}

async fn run_task(
    config: &Config,
    bitcoin_client: impl BitcoinClient,
    stacks_client: LockedClient,
    task: Task,
) -> Event {
    match task {
        Task::CreateAssetContract => deploy_asset_contract(config, stacks_client).await,
        Task::CreateMint(deposit_info) => {
            mint_asset(config, bitcoin_client, stacks_client, deposit_info).await
        }
        Task::CheckBitcoinTransactionStatus(txid) => {
            check_bitcoin_transaction_status(config, txid).await
        }
        Task::CheckStacksTransactionStatus(txid) => {
            check_stacks_transaction_status(stacks_client, txid).await
        }
        Task::FetchBitcoinBlock(block_height) => {
            fetch_bitcoin_block(bitcoin_client, block_height).await
        }
        _ => panic!(),
    }
}

async fn deploy_asset_contract(config: &Config, client: LockedClient) -> Event {
    let contract_bytes = tokio::fs::read_to_string(&config.contract).await.unwrap();

    let public_key =
        StacksPublicKey::from_slice(&config.stacks_credentials.public_key().serialize()).unwrap();

    let tx_auth = TransactionAuth::Standard(
        TransactionSpendingCondition::new_singlesig_p2pkh(public_key).unwrap(),
    );
    let tx_payload = TransactionPayload::SmartContract(
        TransactionSmartContract {
            name: config.contract_name.clone(),
            code_body: StacksString::from_string(&contract_bytes).unwrap(),
        },
        None,
    );

    let tx = StacksTransaction::new(TransactionVersion::Testnet, tx_auth, tx_payload);

    let txid = client
        .lock()
        .await
        .sign_and_broadcast(tx)
        .await
        .expect("Unable to sign and broadcast the asset contract deployment transaction");

    Event::AssetContractBroadcasted(txid)
}

async fn mint_asset(
    config: &Config,
    bitcoin_client: impl BitcoinClient,
    stacks_client: LockedClient,
    deposit_info: DepositInfo,
) -> Event {
    let block = bitcoin_client
        .fetch_block(deposit_info.block_height)
        .await
        .expect("Failed to fetch block");
    let index = block
        .txdata
        .iter()
        .position(|tx| tx.txid() == deposit_info.txid)
        .expect("Failed to find transaction in block");
    let proof_data = ProofData::from_block_and_index(&block, index).to_values();

    let public_key =
        StacksPublicKey::from_slice(&config.stacks_credentials.public_key().serialize()).unwrap();

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
        proof_data.merkle_tree_depth,
        proof_data.block_header,
    ];

    let addr = StacksAddress::consensus_deserialize(&mut Cursor::new(
        config.stacks_credentials.address().serialize_to_vec(),
    ))
    .unwrap();

    let tx_payload = TransactionPayload::ContractCall(TransactionContractCall {
        address: addr,
        contract_name: config.contract_name.clone(),
        function_name: ClarityName::from("mint"),
        function_args,
    });

    let tx = StacksTransaction::new(TransactionVersion::Testnet, tx_auth, tx_payload);

    let txid = stacks_client
        .lock()
        .await
        .sign_and_broadcast(tx)
        .await
        .expect("Unable to sign and broadcast the mint transaction");

    Event::MintBroadcasted(deposit_info, txid)
}

async fn check_bitcoin_transaction_status(_config: &Config, _txid: BitcoinTxId) -> Event {
    todo!();
}

async fn check_stacks_transaction_status(client: LockedClient, txid: StacksTxId) -> Event {
    let status = client
        .lock()
        .await
        .get_transation_status(txid)
        .await
        .expect("Could not get transaction status");

    Event::StacksTransactionUpdate(txid, status)
}

async fn fetch_bitcoin_block(client: impl BitcoinClient, block_height: Option<u32>) -> Event {
    let block_height = if let Some(height) = block_height {
        height
    } else {
        client
            .get_height()
            .await
            .expect("Failed to get bitcoin block height")
    };

    let block = client
        .fetch_block(block_height)
        .await
        .expect("Failed to fetch block");

    Event::BitcoinBlock(block)
}

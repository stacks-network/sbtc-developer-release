//! System

use bdk::bitcoin::Txid as BitcoinTxId;
use blockstack_lib::burnchains::Txid as StacksTxId;
use tokio::fs::File;
use tokio::fs::OpenOptions;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::io::BufWriter;

use blockstack_lib::chainstate::stacks::SinglesigHashMode;
use blockstack_lib::chainstate::stacks::SinglesigSpendingCondition;
use blockstack_lib::chainstate::stacks::StacksTransaction;
use blockstack_lib::chainstate::stacks::TransactionAuth;
use blockstack_lib::chainstate::stacks::TransactionPayload;
use blockstack_lib::chainstate::stacks::TransactionSmartContract;
use blockstack_lib::chainstate::stacks::TransactionSpendingCondition;
use blockstack_lib::chainstate::stacks::TransactionVersion;

use blockstack_lib::util_lib::strings::StacksString;
use blockstack_lib::vm::ContractName;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::debug;

use crate::config::Config;
use crate::event::Event;
use crate::event::TransactionStatus;
use crate::stacks_client::LockedClient;
use crate::stacks_client::StacksClient;
use crate::state;
use crate::task::Task;

/// The main run loop of this system.
/// This function feeds all events to the `state::update` function and spawns all tasks returned from this function.
///
/// The system is bootstrapped by emitting the CreateAssetContract task.
pub async fn run(config: Config) {
    let (tx, mut rx) = mpsc::channel::<Event>(128); // TODO: Make capacity configurable
    let client: LockedClient = StacksClient::new(
        config.stacks_private_key(),
        config.stacks_node_url.clone(),
        reqwest::Client::new(),
        config.private_key.network,
    )
    .into();

    let bootstrap = || {
        spawn(
            config.clone(),
            client.clone(),
            Task::CreateAssetContract,
            tx.clone(),
        );
    };

    let (mut storage, mut state) =
        Storage::load_and_replay(&config, state::State::default(), bootstrap).await;

    // Bootstrap
    spawn(
        config.clone(),
        client.clone(),
        Task::CreateAssetContract,
        tx.clone(),
    );

    while let Some(event) = rx.recv().await {
        storage.record(&event).await;

        let (next_state, tasks) = state::update(&config, state, event);

        for task in tasks {
            spawn(config.clone(), client.clone(), task, tx.clone());
        }

        state = next_state;
    }
}

struct Storage(BufWriter<File>);

impl Storage {
    async fn load_and_replay<F: FnOnce() -> JoinHandle<()>>(
        config: &Config,
        mut state: state::State,
        bootstrap: F,
    ) -> (Self, state::State) {
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .append(true)
            .open(config.state_directory.join("log.ndjson"))
            .await
            .unwrap();

        dbg!(file.metadata().await.unwrap().len());

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

#[tracing::instrument(skip(config, client, result))]
fn spawn(
    config: Config,
    client: LockedClient,
    task: Task,
    result: mpsc::Sender<Event>,
) -> JoinHandle<()> {
    debug!("Spawning task");

    tokio::task::spawn(async move {
        let event = run_task(&config, client, task).await;
        result.send(event).await.expect("Failed to return event");
    })
}

async fn run_task(config: &Config, client: LockedClient, task: Task) -> Event {
    match task {
        Task::CreateAssetContract => deploy_asset_contract(config, client).await,
        Task::CheckBitcoinTransactionStatus(txid) => {
            check_bitcoin_transaction_status(config, txid).await
        }
        Task::CheckStacksTransactionStatus(txid) => {
            check_stacks_transaction_status(config, txid).await
        }
        Task::FetchBitcoinBlock(block_height) => fetch_bitcoin_block(config, block_height).await,
        _ => panic!(),
    }
}

async fn deploy_asset_contract(config: &Config, client: LockedClient) -> Event {
    let contract_bytes = tokio::fs::read_to_string(&config.contract).await.unwrap();

    let tx_auth = TransactionAuth::Standard(
        TransactionSpendingCondition::new_singlesig_p2pkh(config.stacks_public_key()).unwrap(),
    );
    let tx_payload = TransactionPayload::SmartContract(
        TransactionSmartContract {
            name: ContractName::from("sbtc-alpha-romeo321"),
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

    Event::AssetContractCreated(txid)
}

async fn check_bitcoin_transaction_status(_config: &Config, _txid: BitcoinTxId) -> Event {
    todo!();
}

async fn check_stacks_transaction_status(_config: &Config, txid: StacksTxId) -> Event {
    // TODO

    Event::StacksTransactionUpdate(txid, TransactionStatus::Rejected)
}

async fn fetch_bitcoin_block(_config: &Config, _block_height: u64) -> Event {
    todo!();
}

use anyhow::anyhow;
use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::bitcoin::Txid as BitcoinTxId;
use blockstack_lib::address::AddressHashMode;
use blockstack_lib::address::C32_ADDRESS_VERSION_TESTNET_SINGLESIG;
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
use blockstack_lib::chainstate::stacks::StacksTransactionSigner;
use blockstack_lib::chainstate::stacks::TransactionAnchorMode;
use blockstack_lib::chainstate::stacks::TransactionAuth;
use blockstack_lib::chainstate::stacks::TransactionPayload;
use blockstack_lib::chainstate::stacks::TransactionPostConditionMode;
use blockstack_lib::chainstate::stacks::TransactionSmartContract;
use blockstack_lib::chainstate::stacks::TransactionSpendingCondition;
use blockstack_lib::chainstate::stacks::TransactionVersion;
use blockstack_lib::codec::StacksMessageCodec;
use blockstack_lib::core::CHAIN_ID_TESTNET;
use blockstack_lib::types::chainstate::StacksAddress;
use blockstack_lib::types::chainstate::StacksPrivateKey;
use blockstack_lib::types::chainstate::StacksPublicKey;
use blockstack_lib::types::PrivateKey;
use blockstack_lib::util::hash::Hash160;
use blockstack_lib::util_lib::strings::StacksString;
use blockstack_lib::vm::ContractName;
use tokio::sync::mpsc;
use tracing::debug;

use crate::config::Config;
use crate::event::Event;
use crate::event::TransactionStatus;
use crate::state;
use crate::task::Task;

/// The main run loop of this system.
/// This function feeds all events to the `state::update` function and spawns all tasks returned from this function.
///
/// The system is bootstrapped by emitting the CreateAssetContract task.
pub async fn run(config: Config) {
    let (mut storage, mut state) = Storage::load_and_replay(&config, state::State::default()).await;
    let (tx, mut rx) = mpsc::channel::<Event>(128); // TODO: Make capacity configurable

    // Bootstrap
    spawn(config.clone(), Task::CreateAssetContract, tx.clone());

    while let Some(event) = rx.recv().await {
        storage.record(&event).await;

        let (next_state, tasks) = state::update(&config, state, event);

        for task in tasks {
            spawn(config.clone(), task, tx.clone());
        }

        state = next_state;
    }
}

struct Storage(BufWriter<File>);

impl Storage {
    async fn load_and_replay(config: &Config, mut state: state::State) -> (Self, state::State) {
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

#[tracing::instrument(skip(config, result))]
fn spawn(config: Config, task: Task, result: mpsc::Sender<Event>) -> tokio::task::JoinHandle<()> {
    debug!("Spawning task");

    tokio::task::spawn(async move {
        let event = run_task(&config, task).await;
        result.send(event).await.expect("Failed to return event");
    })
}

async fn run_task(config: &Config, task: Task) -> Event {
    match task {
        Task::CreateAssetContract => deploy_asset_contract(config).await,
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

async fn deploy_asset_contract(config: &Config) -> Event {
    // TODO: #73
    println!("Deploying");

    let contract_bytes = tokio::fs::read_to_string(&config.contract).await.unwrap();
    let (private_key, btc_private_key) = get_stacks_private_key(&config.wif).unwrap();

    let mut public_key = StacksPublicKey::from_private(&private_key);
    public_key.set_compressed(true);

    let mut tx = StacksTransaction::new(
        TransactionVersion::Testnet,
        TransactionAuth::Standard(
            TransactionSpendingCondition::new_singlesig_p2pkh(public_key).unwrap(),
        ),
        TransactionPayload::SmartContract(
            TransactionSmartContract {
                name: ContractName::from("sbtc-alpha-romeo123"),
                code_body: StacksString::from_string(&contract_bytes).unwrap(),
            },
            None,
        ),
    );

    tx.set_origin_nonce(120);
    tx.set_tx_fee(2500);

    tx.anchor_mode = TransactionAnchorMode::Any;
    tx.post_condition_mode = TransactionPostConditionMode::Allow;
    tx.chain_id = CHAIN_ID_TESTNET;

    let mut signer = StacksTransactionSigner::new(&mut tx);

    signer.sign_origin(&private_key).unwrap();

    tx = signer.get_tx().unwrap();

    let mut tx_bytes = vec![];
    tx.consensus_serialize(&mut tx_bytes).unwrap();

    std::fs::write("tx.bin", &tx_bytes).unwrap();

    let txid = reqwest::Client::new()
        .post("https://stacks-node-api.testnet.stacks.co/v2/transactions")
        .header("Content-type", "application/octet-stream")
        .body(tx_bytes)
        .send()
        .await
        .unwrap()
        .json::<String>()
        .await
        .unwrap();

    Event::AssetContractCreated(StacksTxId::from_hex(&txid).unwrap())
}

fn get_stacks_private_key(
    wif: &str,
) -> anyhow::Result<(StacksPrivateKey, bdk::bitcoin::PrivateKey)> {
    let pk = bdk::bitcoin::PrivateKey::from_wif(wif)?;

    Ok((
        StacksPrivateKey::from_slice(&pk.to_bytes())
            .map_err(|err| anyhow!("Could not parse stacks private key bytes: {:?}", err))?,
        pk,
    ))
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

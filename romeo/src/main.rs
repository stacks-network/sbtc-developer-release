use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    future::Future,
    path::Path,
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use bdk::bitcoin::{Amount, Block, Txid as BitcoinTxId};
use blockstack_lib::{
    burnchains::Txid as StacksTxId, types::chainstate::StacksAddress,
    vm::types::QualifiedContractIdentifier,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::{
    join,
    sync::broadcast::{self, error::RecvError, Sender},
    task::JoinHandle,
    time::sleep,
};

pub type BlockHeight = u64;

#[derive(Debug, Clone)]
pub struct ContractData(QualifiedContractIdentifier);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Deposit {
    id: BitcoinTxId,
    amount: u64,
    recipient: StacksAddress,
    block_height: BlockHeight,
}

#[derive(Debug, Clone)]
pub struct MintData {
    deposit: Deposit,
    txid: StacksTxId,
}

#[derive(Debug, Clone)]
pub enum Event {
    AssetContractDeployed(ContractData),
    DepositSeen(Deposit),
    MintRequest(Deposit),
    MintBroadcasted(MintData),
    MintConfirmed(MintData),
    MintRejected(MintData),
    WithdrawalSeen,
    BurnCreated,
    BurnBroadcasted,
    BurnConfirmed,
    BurnRejected,
    FulfillmentCreated,
    FulfillmentBroadcasted,
    FulfillmentConfirmed,
    FulfillmentRejected,

    BitcoinBlock(Block),
    NextNonce(u64),
    Tick,
}

trait Actor: Serialize + DeserializeOwned + Default + Send + Sync + 'static {
    const NAME: &'static str;

    fn handle(&mut self, event: Event) -> anyhow::Result<Vec<Event>>;

    fn save(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        serde_json::to_writer_pretty(file, self)?;

        Ok(())
    }

    fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = File::open(path)?;
        let foo = serde_json::from_reader(file)?;

        Ok(foo)
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum DepositState {
    Seen,
    Broadcasted(StacksTxId),
    Rejected(StacksTxId),
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct DepositProcessor {
    block_height: BlockHeight,
    next_nonce: u64,
    deposits: BTreeMap<(BlockHeight, Deposit), DepositState>,
}

impl Actor for DepositProcessor {
    const NAME: &'static str = "DepositProcessor";

    fn handle(&mut self, event: Event) -> anyhow::Result<Vec<Event>> {
        match event {
            Event::BitcoinBlock(block) => {
                self.process_bitcoin_block(block);
                Ok(vec![])
            }
            Event::NextNonce(nonce) => {
                self.next_nonce = nonce;
                Ok(vec![])
            }
            Event::DepositSeen(deposit) => Ok(vec![self.process_deposit(deposit)]),
            Event::MintBroadcasted(data) => {
                self.process_mint_broadcasted(data);
                Ok(vec![])
            }
            Event::MintConfirmed(data) => {
                self.process_mint_confirmed(data);
                Ok(vec![])
            }
            Event::MintRejected(data) => {
                self.process_mint_rejected(data);
                Ok(vec![])
            }
            _ => Ok(vec![]),
        }
    }
}

impl DepositProcessor {
    fn process_bitcoin_block(&mut self, block: Block) {
        self.block_height = block
            .bip34_block_height()
            .expect("Unable to get the Bitcoin block height");
    }

    fn process_deposit(&mut self, deposit: Deposit) -> Event {
        let deposit_not_present = self
            .deposits
            .insert((deposit.block_height, deposit.clone()), DepositState::Seen)
            .is_none();

        assert!(
            deposit_not_present,
            "New deposit already present in the state"
        );

        Event::MintRequest(deposit)
    }

    fn process_mint_broadcasted(&mut self, data: MintData) {
        let MintData { deposit, txid } = data;

        let key = (deposit.block_height, deposit.clone());

        let state = self
            .deposits
            .get_mut(&key)
            .expect("Broadcasted deposit should be in the map");

        assert!(
            matches!(state, DepositState::Seen),
            "Broadcasted deposit not in the expected state: {:?}",
            state
        );

        *state = DepositState::Broadcasted(txid);
    }

    fn process_mint_confirmed(&mut self, data: MintData) {
        let MintData { deposit, txid } = data;

        let key = (deposit.block_height, deposit.clone());

        let state = self
            .deposits
            .get_mut(&key)
            .expect("Confirmed deposit should be in the map");

        assert!(
            matches!(state, DepositState::Broadcasted(_)),
            "Confirmed deposit not in the expected state: {:?}",
            state
        );

        self.deposits
            .remove(&key)
            .expect("Confirmed deposit was not in state");
    }

    fn process_mint_rejected(&mut self, data: MintData) {
        let MintData { deposit, txid } = data;

        let key = (deposit.block_height, deposit.clone());

        let state = self
            .deposits
            .get_mut(&key)
            .expect("Rejected deposit should be in the map");

        assert!(
            matches!(state, DepositState::Broadcasted(_)),
            "Rejected deposit not in the expected state: {:?}",
            state
        );

        *state = DepositState::Rejected(txid);
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ContractManager {
    deployed: bool,
    contract: Option<QualifiedContractIdentifier>,
}

impl Actor for ContractManager {
    const NAME: &'static str = "ContractManager";
    fn handle(&mut self, event: Event) -> anyhow::Result<Vec<Event>> {
        match event {
            Event::Tick => {
                if self.deployed {
                    return Ok(vec![]);
                }

                // todo!("Deploy the contract");
                self.deployed = true;

                return Ok(vec![]);

                // let contract = todo!();
                // self.contract = Some(contract);

                // Ok(vec![Event::AssetContractDeployed(ContractData(contract))])
            }
            _ => Ok(vec![]),
        }
    }
}

fn spawn_actor<A: Actor>(sender: &Sender<Event>) -> JoinHandle<()> {
    let mut actor = A::load(".").unwrap_or_default();

    let sender = sender.clone();
    let mut receiver = sender.subscribe();

    tokio::spawn(async move {
        loop {
            let new_events = match receiver.recv().await {
                Ok(event) => {
                    let new_events = actor.handle(event).unwrap();

                    let save_file = format!("./{}.json", A::NAME);
                    actor.save(save_file).unwrap();

                    new_events
                }
                Err(RecvError::Closed) => break,
                _ => vec![],
            };

            for event in new_events {
                sender.send(event).unwrap();
            }
        }
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (sender, _) = broadcast::channel::<Event>(128);

    let sender_clone = sender.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(10)).await;
        println!("Sending tick");
        sender_clone.send(Event::Tick).unwrap();
    });

    let h1 = spawn_actor::<DepositProcessor>(&sender);
    let h2 = spawn_actor::<ContractManager>(&sender);

    let _ = join!(h1, h2);

    Ok(())
}

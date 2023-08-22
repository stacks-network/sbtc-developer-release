use std::collections::BTreeMap;

use bdk::bitcoin::Block;
use blockstack_lib::burnchains::Txid as StacksTxId;
use serde::{Deserialize, Serialize};

use crate::actor::Actor;
use crate::event;
use crate::event::Event;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DepositProcessor {
    block_height: BlockHeight,
    next_nonce: u64,
    deposits: BTreeMap<(BlockHeight, event::Deposit), DepositState>,
}

type BlockHeight = u64;

#[derive(Debug, Serialize, Deserialize)]
enum DepositState {
    Seen,
    Broadcasted(StacksTxId),
    Rejected(StacksTxId),
}

impl Actor for DepositProcessor {
    const NAME: &'static str = "deposit_processor";

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

    fn process_deposit(&mut self, deposit: event::Deposit) -> Event {
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

    fn process_mint_broadcasted(&mut self, data: event::MintData) {
        let event::MintData { deposit, txid } = data;

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

    fn process_mint_confirmed(&mut self, data: event::MintData) {
        let event::MintData { deposit, .. } = data;

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

    fn process_mint_rejected(&mut self, data: event::MintData) {
        let event::MintData { deposit, txid } = data;

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

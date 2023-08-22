use bdk::bitcoin::{Block, Txid as BitcoinTxId};
use blockstack_lib::{
    burnchains::Txid as StacksTxId, types::chainstate::StacksAddress,
    vm::types::QualifiedContractIdentifier,
};
use serde::{Deserialize, Serialize};

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

pub type BlockHeight = u64;

#[derive(Debug, Clone)]
pub struct ContractData(QualifiedContractIdentifier);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Deposit {
    pub id: BitcoinTxId,
    pub amount: u64,
    pub recipient: StacksAddress,
    pub block_height: BlockHeight,
}

#[derive(Debug, Clone)]
pub struct MintData {
    pub deposit: Deposit,
    pub txid: StacksTxId,
}

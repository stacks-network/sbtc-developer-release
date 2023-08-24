use bdk::bitcoin::{Block, Txid as BitcoinTxId};
use blockstack_lib::burnchains::Txid as StacksTxId;

#[derive(Debug, Clone)]
pub enum Event {
    MintCreated(StacksTxId),
    BurnCreated(StacksTxId),
    FulfillCreated(BitcoinTxId),
    AssetContractCreated(StacksTxId),

    StacksTransactionUpdate(StacksTxId, TransactionStatus),
    BitcoinTransactionUpdate(BitcoinTxId, TransactionStatus),

    BitcoinBlock(Block),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TransactionStatus {
    Broadcasted,
    Confirmed,
    Rejected,
}

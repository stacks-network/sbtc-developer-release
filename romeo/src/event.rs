use bdk::bitcoin::{Block, Txid as BitcoinTxId};
use blockstack_lib::burnchains::Txid as StacksTxId;

#[derive(Debug, Clone)]
pub enum Event {
    StacksTransactionUpdate(StacksTxId, TransactionStatus),
    BitcoinTransactionUpdate(BitcoinTxId, TransactionStatus),

    BitcoinBlock(Block),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TransactionStatus {
    Created,
    Broadcasted,
    Confirmed,
    Rejected,
}

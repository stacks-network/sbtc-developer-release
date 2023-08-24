use bdk::bitcoin::{Transaction as BitcoinTransaction, Txid as BitcoinTxId};
use blockstack_lib::{burnchains::Txid as StacksTxId, chainstate::stacks::StacksTransaction};

pub enum Task {
    BroadcastBitcoinTransaction(BitcoinTransaction),
    BroadcastStacksTransaction(StacksTransaction),
    CheckBitcoinTransactionStatus(BitcoinTxId),
    CheckStacksTransactionStatus(StacksTxId),

    FetchBitcoinBlock,
}

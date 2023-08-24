use bdk::bitcoin::{Block, Txid as BitcoinTxId};
use blockstack_lib::burnchains::Txid as StacksTxId;

use crate::state::DepositInfo;
use crate::state::WithdrawalInfo;

#[derive(Debug, Clone)]
pub enum Event {
    MintCreated(DepositInfo, StacksTxId),
    BurnCreated(WithdrawalInfo, StacksTxId),
    FulfillCreated(WithdrawalInfo, BitcoinTxId),
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

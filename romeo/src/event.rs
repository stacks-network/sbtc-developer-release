use bdk::bitcoin::{Block, Txid as BitcoinTxId};
use blockstack_lib::burnchains::Txid as StacksTxId;

use crate::state::DepositInfo;
use crate::state::WithdrawalInfo;

/// Events are spawned from tasks and used
/// to update the system state.
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

/// Status of a broadcasted transaction, useful for implementing retry logic
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TransactionStatus {
    Broadcasted,
    Confirmed,
    Rejected,
}

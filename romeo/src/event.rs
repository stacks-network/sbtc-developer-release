//! Event

use bdk::bitcoin::{Block, Txid as BitcoinTxId};
use blockstack_lib::burnchains::Txid as StacksTxId;

use crate::state::DepositInfo;
use crate::state::WithdrawalInfo;

/// Events are spawned from tasks and used
/// to update the system state.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Event {
    /// A mint transaction has been created and broadcasted
    MintCreated(DepositInfo, StacksTxId),

    /// A burn transaction has been created and broadcasted
    BurnCreated(WithdrawalInfo, StacksTxId),

    /// A fulfill transaction has been created and broadcasted
    FulfillCreated(WithdrawalInfo, BitcoinTxId),

    /// The asset contract deploy transaction has been created and broadcasted
    AssetContractCreated(StacksTxId),

    /// A stacks node has responded with an updated status regarding this txid
    StacksTransactionUpdate(StacksTxId, TransactionStatus),

    /// A bitcoin node has responded with an updated status regarding this txid
    BitcoinTransactionUpdate(BitcoinTxId, TransactionStatus),

    /// A wild bitcoin block has appeared
    BitcoinBlock(Block),
}

/// Status of a broadcasted transaction, useful for implementing retry logic
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum TransactionStatus {
    /// This transaction has been broadcasted to a node
    Broadcasted,
    /// This transaction has received `Config::number_of_required_confirmations` confirmations
    Confirmed,
    /// There are indications that this transaction will never be mined
    Rejected,
}

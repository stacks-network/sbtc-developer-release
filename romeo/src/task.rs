use bdk::bitcoin::Txid as BitcoinTxId;
use blockstack_lib::burnchains::Txid as StacksTxId;

use crate::state;

/// Represents I/O operations performed by the system
#[derive(Debug)]
pub enum Task {
    /// Create and broadcast a mint stacks transaction
    CreateMint(state::DepositInfo),
    /// Create and broadcast a burn stacks transaction
    CreateBurn(state::WithdrawalInfo),
    /// Create and broadcast a fulfill bitcoin transaction
    CreateFulfill(state::WithdrawalInfo),

    /// Create and broadcast the asset contract deploy transaction
    CreateAssetContract,

    /// Poll a bitcoin node for the status of a transaction
    CheckBitcoinTransactionStatus(BitcoinTxId),

    /// Poll a stacks node for the status of a transaction
    CheckStacksTransactionStatus(StacksTxId),

    /// Fetch a bitcoin block for the given block height from the current canonical bitcoin fork
    FetchBitcoinBlock(u64),
}

use bdk::bitcoin::{Transaction as BitcoinTransaction, Txid as BitcoinTxId};
use blockstack_lib::{burnchains::Txid as StacksTxId, chainstate::stacks::StacksTransaction};

use crate::state;

pub enum Task {
    CreateMint(state::DepositInfo),
    CreateBurn,
    CreateFulfill,
    CreateAssetContract,

    CheckBitcoinTransactionStatus(BitcoinTxId),
    CheckStacksTransactionStatus(StacksTxId),

    FetchBitcoinBlock(u64),
}

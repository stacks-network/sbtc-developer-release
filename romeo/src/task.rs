use bdk::bitcoin::Txid as BitcoinTxId;
use blockstack_lib::burnchains::Txid as StacksTxId;

use crate::state;

#[derive(Debug)]
pub enum Task {
    CreateMint(state::DepositInfo),
    CreateBurn(state::WithdrawalInfo),
    CreateFulfill(state::WithdrawalInfo),
    CreateAssetContract,

    CheckBitcoinTransactionStatus(BitcoinTxId),
    CheckStacksTransactionStatus(StacksTxId),

    FetchBitcoinBlock(u64),
}

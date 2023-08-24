use bdk::bitcoin::Txid as BitcoinTxId;
use blockstack_lib::burnchains::Txid as StacksTxId;

pub enum Task {
    Mint,
    Burn,
    Fulfill,

    FetchBitcoinBlock,

    BroadcastBitcoinTransaction,
    BroadcastStacksTransaction,
    CheckBitcoinTransactionStatus(BitcoinTxId),
    CheckStacksTransactionStatus(StacksTxId),

    DeployAssetContract,
}

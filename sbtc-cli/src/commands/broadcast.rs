use std::io::stdout;

use bdk::{
    bitcoin::{
        psbt::serialize::{Deserialize, Serialize},
        Network, Transaction,
    },
    blockchain::Blockchain,
};
use clap::Parser;

use crate::commands::utils;

#[derive(Parser, Debug, Clone)]
pub struct BroadcastArgs {
    /// The network to broadcast to
    #[clap(short, long, default_value_t = Network::Testnet)]
    network: Network,
    /// The transaction to broadcast
    tx: String,
}

pub fn broadcast_tx(broadcast: &BroadcastArgs) -> anyhow::Result<()> {
    let blockchain = utils::init_blockstream_blockchain()?;
    let tx = Transaction::deserialize(&hex::decode(&broadcast.tx)?)?;
    blockchain.broadcast(&tx)?;

    serde_json::to_writer_pretty(
        stdout(),
        &utils::TransactionData {
            tx_id: tx.txid().to_string(),
            tx_hex: hex::encode(tx.serialize()),
        },
    )?;

    Ok(())
}

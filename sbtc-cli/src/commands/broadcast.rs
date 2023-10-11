use std::io::stdout;

use bdk::{
	bitcoin::{psbt::serialize::Deserialize, Transaction},
	electrum_client::ElectrumApi,
};
use clap::Parser;
use url::Url;

#[derive(Parser, Debug, Clone)]
pub struct BroadcastArgs {
	/// Where to broadcast the transaction
	pub node_url: Url,

	/// The transaction to broadcast
	pub tx: String,
}

pub fn broadcast_tx(broadcast: &BroadcastArgs) -> anyhow::Result<()> {
	let client =
		bdk::electrum_client::Client::new(broadcast.node_url.as_str())?;
	let tx = Transaction::deserialize(&hex::decode(&broadcast.tx)?)?;

	client.transaction_broadcast(&tx)?;
	serde_json::to_writer_pretty(stdout(), &tx.txid().to_string())?;

	Ok(())
}

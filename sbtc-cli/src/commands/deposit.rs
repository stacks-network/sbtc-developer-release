use std::{io::stdout, str::FromStr};

use bdk::{
	bitcoin::{
		psbt::serialize::Serialize, Address as BitcoinAddress,
		Network as BitcoinNetwork, PrivateKey,
	},
	blockchain::{
		ConfigurableBlockchain, ElectrumBlockchain, ElectrumBlockchainConfig,
	},
	database::MemoryDatabase,
	template::P2Wpkh,
	SyncOptions, Wallet,
};
use clap::Parser;
use sbtc_core::operations::op_return::deposit::build_deposit_transaction;
use stacks_core::utils::PrincipalData;
use url::Url;

use crate::commands::utils;

#[derive(Parser, Debug, Clone)]
pub struct DepositArgs {
	/// Where to broadcast the transaction
	#[clap(short('u'), long)]
	node_url: Url,

	/// Bitcoin WIF of the P2wPKH address
	#[clap(short, long)]
	wif: String,

	/// Bitcoin network where the deposit will be broadcasted to
	#[clap(short, long)]
	network: BitcoinNetwork,

	/// Stacks address that will receive sBTC
	#[clap(short, long)]
	recipient: String,

	/// The amount of sats to send
	#[clap(short, long)]
	amount: u64,

	/// Bitcoin address of the sbtc wallet
	#[clap(short, long)]
	sbtc_wallet: String,
}

pub fn build_deposit_tx(deposit: &DepositArgs) -> anyhow::Result<()> {
	let private_key = PrivateKey::from_wif(&deposit.wif)?;

	let blockchain =
		ElectrumBlockchain::from_config(&ElectrumBlockchainConfig {
			url: deposit.node_url.as_str().to_string(),
			socks5: None,
			retry: 3,
			timeout: Some(10),
			stop_gap: 10,
			validate_domain: false,
		})?;

	let wallet = Wallet::new(
		P2Wpkh(private_key),
		Some(P2Wpkh(private_key)),
		deposit.network,
		MemoryDatabase::default(),
	)?;

	wallet.sync(&blockchain, SyncOptions::default())?;

	let stx_recipient = PrincipalData::try_from(deposit.recipient.to_string())?;
	let sbtc_wallet_address = BitcoinAddress::from_str(&deposit.sbtc_wallet)?;

	let tx = build_deposit_transaction(
		wallet,
		stx_recipient,
		sbtc_wallet_address,
		deposit.amount,
		deposit.network,
	)?;

	serde_json::to_writer_pretty(
		stdout(),
		&utils::TransactionData {
			id: tx.txid().to_string(),
			hex: hex::encode(tx.serialize()),
		},
	)?;

	Ok(())
}

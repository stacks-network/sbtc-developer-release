use std::str::FromStr;

use bdk::{
	bitcoin::{
		Address as BitcoinAddress, Network as BitcoinNetwork, PrivateKey,
		Transaction,
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
use stacks_core::address::StacksAddress;
use url::Url;

#[derive(Parser, Debug, Clone)]
pub struct DepositArgs {
	/// Where to broadcast the transaction
	#[clap(short('u'), long)]
	pub node_url: Url,

	/// Bitcoin WIF of the P2wPKH address
	#[clap(short, long)]
	pub wif: String,

	/// Bitcoin network where the deposit will be broadcasted to
	#[clap(short, long)]
	pub network: BitcoinNetwork,

	/// Stacks address that will receive sBTC
	#[clap(short, long)]
	pub recipient: String,

	/// The amount of sats to send
	#[clap(short, long)]
	pub amount: u64,

	/// Bitcoin address of the sbtc wallet
	#[clap(short, long)]
	pub sbtc_wallet: String,
}

pub fn build_deposit_tx(deposit: &DepositArgs) -> anyhow::Result<Transaction> {
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

	let recipient_address =
		StacksAddress::try_from(deposit.recipient.as_str())?;
	let sbtc_wallet_address = BitcoinAddress::from_str(&deposit.sbtc_wallet)?;

	build_deposit_transaction(
		wallet,
		recipient_address.into(),
		sbtc_wallet_address,
		deposit.amount,
		deposit.network,
	)
	.map_err(|e| e.into())
}

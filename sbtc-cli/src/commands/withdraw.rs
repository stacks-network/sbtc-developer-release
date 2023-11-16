use std::str::FromStr;

use bdk::{
	bitcoin::{
		blockdata::transaction::Transaction, Address as BitcoinAddress,
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
use url::Url;

#[derive(Parser, Debug, Clone)]
pub struct WithdrawalArgs {
	/// Where to broadcast the transaction
	#[clap(short('u'), long)]
	pub node_url: Url,

	/// Bitcoin network where the deposit will be broadcasted to
	#[clap(short, long)]
	pub network: BitcoinNetwork,

	/// WIF of the Bitcoin P2WPKH address that will broadcast and pay for the
	/// withdrawal request
	#[clap(short, long)]
	pub wif: String,

	/// WIF of the Stacks address that owns sBTC to be withdrawn
	#[clap(short, long)]
	pub drawee_wif: String,

	/// Bitcoin address that will receive BTC
	#[clap(short('b'), long)]
	pub payee_address: String,

	/// The amount of sats to withdraw
	#[clap(short, long)]
	pub amount: u64,

	/// The amount of sats to send for the fulfillment fee
	#[clap(short, long)]
	pub fulfillment_fee: u64,

	/// Bitcoin address of the sbtc wallet
	#[clap(short, long)]
	pub sbtc_wallet: String,
}

pub fn build_withdrawal_tx(
	withdrawal: &WithdrawalArgs,
) -> anyhow::Result<Transaction> {
	let private_key = PrivateKey::from_wif(&withdrawal.wif)?;

	let blockchain =
		ElectrumBlockchain::from_config(&ElectrumBlockchainConfig {
			url: withdrawal.node_url.as_str().to_string(),
			socks5: None,
			retry: 3,
			timeout: Some(10),
			stop_gap: 10,
			validate_domain: false,
		})?;

	let wallet = Wallet::new(
		P2Wpkh(private_key),
		Some(P2Wpkh(private_key)),
		withdrawal.network,
		MemoryDatabase::default(),
	)?;

	wallet.sync(&blockchain, SyncOptions::default())?;

	let drawee_stacks_private_key =
		PrivateKey::from_wif(&withdrawal.drawee_wif)?.inner;
	let payee_bitcoin_address =
		BitcoinAddress::from_str(&withdrawal.payee_address)?;
	let sbtc_wallet_bitcoin_address =
		BitcoinAddress::from_str(&withdrawal.sbtc_wallet)?;

	sbtc_core::operations::op_return::withdrawal_request::build_withdrawal_tx(
		&wallet,
		withdrawal.network,
		drawee_stacks_private_key,
		payee_bitcoin_address,
		sbtc_wallet_bitcoin_address,
		withdrawal.amount,
		withdrawal.fulfillment_fee,
	)
	.map_err(|e| e.into())
}

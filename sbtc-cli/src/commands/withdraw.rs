use std::{io::stdout, str::FromStr};

use bdk::{
    bitcoin::{
        psbt::serialize::Serialize, Address as BitcoinAddress, Network as BitcoinNetwork,
        PrivateKey,
    },
    blockchain::{ConfigurableBlockchain, ElectrumBlockchain, ElectrumBlockchainConfig},
    database::MemoryDatabase,
    template::P2Wpkh,
    SyncOptions, Wallet,
};
use clap::Parser;
use url::Url;

use crate::commands::utils::TransactionData;

#[derive(Parser, Debug, Clone)]
pub struct WithdrawalArgs {
	/// Where to broadcast the transaction
	#[clap(short('u'), long)]
	node_url: Url,

    /// Bitcoin network where the deposit will be broadcasted to
    #[clap(short, long)]
    network: BitcoinNetwork,

    /// WIF of the Bitcoin P2WPKH address that will broadcast and pay for the withdrawal request
    #[clap(short, long)]
    wif: String,

    /// WIF of the Stacks address that owns sBTC to be withdrawn
    #[clap(short, long)]
    drawee_wif: String,

    /// Bitcoin address that will receive BTC
    #[clap(short('b'), long)]
    payee_address: String,

    /// The amount of sats to withdraw
    #[clap(short, long)]
    amount: u64,

    /// The amount of sats to send for the fulfillment fee
    #[clap(short, long)]
    fulfillment_fee: u64,

    /// Bitcoin address of the peg wallet
    #[clap(short, long)]
    peg_wallet: String,
}

pub fn build_withdrawal_tx(withdrawal: &WithdrawalArgs) -> anyhow::Result<()> {
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

    let broadcaster_bitcoin_private_key = PrivateKey::from_wif(&withdrawal.wif)?;
    let drawee_stacks_private_key = PrivateKey::from_wif(&withdrawal.drawee_wif)?.inner;
    let payee_bitcoin_address = BitcoinAddress::from_str(&withdrawal.payee_address)?;
    let peg_wallet_bitcoin_address = BitcoinAddress::from_str(&withdrawal.peg_wallet)?;

    let tx = sbtc_core::operations::op_return::withdrawal_request::build_withdrawal_tx(
        &wallet,
        broadcaster_bitcoin_private_key,
        drawee_stacks_private_key,
        payee_bitcoin_address,
        peg_wallet_bitcoin_address,
        withdrawal.amount,
        withdrawal.fulfillment_fee,
    )?;

    serde_json::to_writer_pretty(
        stdout(),
        &TransactionData {
            id: tx.txid().to_string(),
            hex: hex::encode(tx.serialize()),
        },
    )?;

	Ok(())
}

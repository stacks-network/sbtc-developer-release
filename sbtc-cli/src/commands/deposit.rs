use std::{io::stdout, str::FromStr};

use bdk::bitcoin::{psbt::serialize::Serialize, Address as BitcoinAddress, PrivateKey};
use clap::Parser;
use sbtc_core::operations::op_return::deposit::build_deposit_transaction;
use stacks_core::address::StacksAddress;

use crate::{commands::utils, config::read_config};

#[derive(Parser, Debug, Clone)]
pub struct DepositArgs {
    /// P2WPKH BTC private key in WIF format
    #[clap(short, long)]
    wif: String,

    /// Stacks address that will receive sBTC
    #[clap(short, long)]
    recipient: String,

    /// The amount of sats to send
    #[clap(short, long)]
    amount: u64,

    /// Dkg wallet address
    #[clap(short, long)]
    dkg_wallet: String,
}

pub fn build_deposit_tx(deposit: &DepositArgs) -> anyhow::Result<()> {
    let config = read_config().expect("Could not read sbtc config, did you try `sbtc init`?");
    let private_key = PrivateKey::from_wif(&deposit.wif)?;
    let wallet = utils::setup_wallet_from_config(&config, private_key)?;
    let recipient_address = StacksAddress::try_from(deposit.recipient.as_str())?;
    let dkg_address = BitcoinAddress::from_str(&deposit.dkg_wallet)?;

    let tx = build_deposit_transaction(
        wallet,
        recipient_address.into(),
        dkg_address,
        deposit.amount,
        private_key.network,
    )?;

    serde_json::to_writer_pretty(
        stdout(),
        &utils::TransactionData {
            tx_id: tx.txid().to_string(),
            tx_hex: hex::encode(tx.serialize()),
        },
    )?;

    Ok(())
}

use std::{io::stdout, iter::once, str::FromStr};

use anyhow::anyhow;
use bdk::{
    bitcoin::{
        psbt::{serialize::Serialize, PartiallySignedTransaction},
        Address as BitcoinAddress, Network, PrivateKey,
    },
    database::MemoryDatabase,
    SignOptions, Wallet,
};
use clap::Parser;
use stacks_core::{address::StacksAddress, crypto::Hashing};

use crate::commands::utils;

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
    let private_key = PrivateKey::from_wif(&deposit.wif)?;
    let wallet = utils::setup_wallet(private_key)?;
    let recipient = StacksAddress::try_from(deposit.recipient.as_str())?;
    let dkg_address = BitcoinAddress::from_str(&deposit.dkg_wallet)?;

    let mut psbt = deposit_psbt(
        &wallet,
        &recipient,
        &dkg_address,
        deposit.amount,
        &private_key.network,
    )?;

    wallet.sign(&mut psbt, SignOptions::default())?;
    let tx = psbt.extract_tx();

    serde_json::to_writer_pretty(
        stdout(),
        &utils::TransactionData {
            tx_id: tx.txid().to_string(),
            tx_hex: array_bytes::bytes2hex("", tx.serialize()),
        },
    )?;

    Ok(())
}

fn deposit_psbt(
    wallet: &Wallet<MemoryDatabase>,
    recipient: &StacksAddress,
    dkg_address: &BitcoinAddress,
    amount: u64,
    network: &Network,
) -> anyhow::Result<PartiallySignedTransaction> {
    let mut tx_builder = wallet.build_tx();

    let op_return_script = utils::build_op_return_script(&deposit_data(recipient, network));
    let dkg_script = dkg_address.script_pubkey();
    let dust_amount = dkg_script.dust_value().to_sat();

    if amount < dust_amount {
        return Err(anyhow!(
            "Provided amount {} is less than the dust amount: {}",
            amount,
            dust_amount
        ));
    }

    let outputs = [(op_return_script, 0), (dkg_script, amount)];

    for (script, amount) in outputs.clone() {
        tx_builder.add_recipient(script, amount);
    }

    let (mut partial_tx, _) = tx_builder.finish()?;

    partial_tx.unsigned_tx.output =
        utils::reorder_outputs(partial_tx.unsigned_tx.output.into_iter(), outputs);

    Ok(partial_tx)
}

fn deposit_data(recipient: &StacksAddress, network: &Network) -> Vec<u8> {
    utils::magic_bytes(network)
        .into_iter()
        .chain(once(b'<'))
        .chain(once(recipient.version() as u8))
        .chain(recipient.hash().as_bytes().to_vec())
        .collect()
}

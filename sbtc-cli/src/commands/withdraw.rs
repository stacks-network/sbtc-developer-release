use std::{io::stdout, iter::once, str::FromStr};

use anyhow::anyhow;
use bdk::{
    bitcoin::{
        psbt::{serialize::Serialize, PartiallySignedTransaction},
        secp256k1::{Message, Secp256k1},
        Address as BitcoinAddress, Network, PrivateKey,
    },
    database::MemoryDatabase,
    SignOptions, Wallet,
};
use clap::Parser;
use stacks_core::crypto::{sha256::Sha256Hasher, Hashing};

use crate::commands::utils::TransactionData;
use crate::commands::utils::{build_op_return_script, magic_bytes, reorder_outputs, setup_wallet};

#[derive(Parser, Debug, Clone)]
pub struct WithdrawalArgs {
    /// P2WPKH BTC private key in WIF format
    #[clap(short, long)]
    wif: String,

    /// P2WPKH sBTC sender private key in WIF format
    #[clap(short, long)]
    sender_wif: String,

    /// Bitcoin address that will receive BTC
    #[clap(short, long)]
    recipient: String,

    /// The amount of sats to send
    #[clap(short, long)]
    amount: u64,

    /// The amount of sats to send as the fulfillment fee
    #[clap(short, long)]
    fulfillment_fee: u64,

    /// Dkg wallet address
    #[clap(short, long)]
    dkg_wallet: String,
}

pub fn build_withdrawal_tx(withdrawal: &WithdrawalArgs) -> anyhow::Result<()> {
    let private_key = PrivateKey::from_wif(&withdrawal.wif)?;

    let wallet = setup_wallet(private_key)?;

    let sender_private_key = PrivateKey::from_wif(&withdrawal.sender_wif)?;
    let recipient = BitcoinAddress::from_str(&withdrawal.recipient)?;
    let dkg_address = BitcoinAddress::from_str(&withdrawal.dkg_wallet)?;

    let mut psbt = withdrawal_psbt(
        &wallet,
        &sender_private_key,
        &recipient,
        &dkg_address,
        withdrawal.amount,
        withdrawal.fulfillment_fee,
        &private_key.network,
    )?;

    wallet.sign(&mut psbt, SignOptions::default())?;
    let tx = psbt.extract_tx();

    serde_json::to_writer_pretty(
        stdout(),
        &TransactionData {
            tx_id: tx.txid().to_string(),
            tx_hex: array_bytes::bytes2hex("", tx.serialize()),
        },
    )?;

    Ok(())
}

fn withdrawal_psbt(
    wallet: &Wallet<MemoryDatabase>,
    sender_private_key: &PrivateKey,
    recipient: &BitcoinAddress,
    dkg_address: &BitcoinAddress,
    amount: u64,
    fulfillment_fee: u64,
    network: &Network,
) -> anyhow::Result<PartiallySignedTransaction> {
    let recipient_script = recipient.script_pubkey();
    let dkg_wallet_script = dkg_address.script_pubkey();

    // Check that we have enough to cover dust
    let recipient_dust_amount = recipient_script.dust_value().to_sat();
    let dkg_wallet_dust_amount = dkg_wallet_script.dust_value().to_sat();

    if fulfillment_fee < dkg_wallet_dust_amount {
        return Err(anyhow!(
            "Provided fulfillment fee {} is less than the dust amount: {}",
            fulfillment_fee,
            dkg_wallet_dust_amount
        ));
    }

    let op_return_script = build_op_return_script(&withdrawal_data(
        recipient,
        amount,
        sender_private_key,
        network,
    ));

    let mut tx_builder = wallet.build_tx();

    let outputs = [
        (op_return_script, 0),
        (recipient_script, recipient_dust_amount),
        (dkg_wallet_script, fulfillment_fee),
    ];

    for (script, amount) in outputs.clone() {
        tx_builder.add_recipient(script, amount);
    }

    let (mut partial_tx, _) = tx_builder.finish()?;

    partial_tx.unsigned_tx.output =
        reorder_outputs(partial_tx.unsigned_tx.output.into_iter(), outputs);

    Ok(partial_tx)
}

fn withdrawal_data(
    recipient: &BitcoinAddress,
    amount: u64,
    sender_private_key: &PrivateKey,
    network: &Network,
) -> Vec<u8> {
    let mut msg = amount.to_be_bytes().to_vec();
    msg.extend_from_slice(recipient.script_pubkey().as_bytes());

    let msg_hash = Sha256Hasher::new(msg.as_slice());
    let msg_hash_bytes = array_bytes::hex2bytes(msg_hash).unwrap();
    let msg_ecdsa = Message::from_slice(&msg_hash_bytes).unwrap();

    let (recovery_id, signature) = Secp256k1::new()
        .sign_ecdsa_recoverable(&msg_ecdsa, &sender_private_key.inner)
        .serialize_compact();

    magic_bytes(network)
        .into_iter()
        .chain(once(b'>'))
        .chain(amount.to_be_bytes())
        .chain(once(recovery_id.to_i32() as u8))
        .chain(signature)
        .collect()
}

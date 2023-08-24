use std::iter::once;

use bdk::{database::MemoryDatabase, SignOptions, Wallet};
use bitcoin::{
    psbt::PartiallySignedTransaction,
    secp256k1::{Message, Secp256k1},
    Address as BitcoinAddress, Network, PrivateKey, Transaction,
};
use stacks_core::crypto::{sha256::Sha256Hasher, Hashing};

use crate::{
    operations::construction::utils::{
        build_op_return_script, magic_bytes, reorder_outputs, setup_wallet,
    },
    SBTCError, SBTCResult,
};

/// Construct a BTC transaction containing the provided sBTC withdrawal data
pub fn build_withdrawal_tx(
    withdrawer_bitcoin_private_key: PrivateKey,
    withdrawer_stacks_private_key: PrivateKey,
    receiver_address: BitcoinAddress,
    amount: u64,
    fulfillment_fee: u64,
    dkg_address: BitcoinAddress,
) -> SBTCResult<Transaction> {
    let wallet = setup_wallet(withdrawer_bitcoin_private_key)?;

    let mut psbt = withdrawal_psbt(
        &wallet,
        &withdrawer_stacks_private_key,
        &receiver_address,
        &dkg_address,
        amount,
        fulfillment_fee,
        withdrawer_bitcoin_private_key.network,
    )?;

    wallet
        .sign(&mut psbt, SignOptions::default())
        .map_err(|err| SBTCError::BDKError("Could not sign withdrawal transaction", err))?;

    Ok(psbt.extract_tx())
}

fn withdrawal_psbt(
    wallet: &Wallet<MemoryDatabase>,
    sender_private_key: &PrivateKey,
    recipient: &BitcoinAddress,
    dkg_address: &BitcoinAddress,
    amount: u64,
    fulfillment_fee: u64,
    network: Network,
) -> SBTCResult<PartiallySignedTransaction> {
    let recipient_script = recipient.script_pubkey();
    let dkg_wallet_script = dkg_address.script_pubkey();

    // Check that we have enough to cover dust
    let recipient_dust_amount = recipient_script.dust_value().to_sat();
    let dkg_wallet_dust_amount = dkg_wallet_script.dust_value().to_sat();

    if fulfillment_fee < dkg_wallet_dust_amount {
        return Err(SBTCError::AmountInsufficient(
            fulfillment_fee,
            dkg_wallet_dust_amount,
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

    let (mut partial_tx, _) = tx_builder.finish().map_err(|err| {
        SBTCError::BDKError(
            "Could not build partially signed withdrawal transaction",
            err,
        )
    })?;

    partial_tx.unsigned_tx.output = reorder_outputs(partial_tx.unsigned_tx.output, outputs);

    Ok(partial_tx)
}

fn withdrawal_data(
    recipient: &BitcoinAddress,
    amount: u64,
    sender_private_key: &PrivateKey,
    network: Network,
) -> Vec<u8> {
    let mut msg = amount.to_be_bytes().to_vec();
    msg.extend_from_slice(recipient.script_pubkey().as_bytes());

    let msg_hash = Sha256Hasher::new(msg);
    let msg_ecdsa = Message::from_slice(msg_hash.as_ref()).unwrap();

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

use std::iter::once;

use bdk::{database::MemoryDatabase, SignOptions, Wallet};
use bitcoin::{
    psbt::PartiallySignedTransaction, Address as BitcoinAddress, Network, PrivateKey, Transaction,
};
use stacks_core::address::StacksAddress;

use crate::{
    operations::construction::utils::{
        build_op_return_script, magic_bytes, reorder_outputs, setup_wallet,
    },
    SBTCError, SBTCResult,
};

pub fn deposit(
    depositor_private_key: PrivateKey,
    recipient_address: &StacksAddress,
    amount: u64,
    dkg_address: &BitcoinAddress,
) -> SBTCResult<Transaction> {
    let wallet = setup_wallet(depositor_private_key)?;

    let mut psbt = create_partially_signed_deposit_transaction(
        &wallet,
        recipient_address,
        dkg_address,
        amount,
        depositor_private_key.network,
    )?;

    wallet
        .sign(&mut psbt, SignOptions::default())
        .map_err(|err| SBTCError::BDKError("Could not sign transaction", err))?;

    Ok(psbt.extract_tx())
}

fn create_partially_signed_deposit_transaction(
    wallet: &Wallet<MemoryDatabase>,
    recipient: &StacksAddress,
    dkg_address: &BitcoinAddress,
    amount: u64,
    network: Network,
) -> SBTCResult<PartiallySignedTransaction> {
    let mut tx_builder = wallet.build_tx();

    let op_return_script = build_op_return_script(&deposit_data(recipient, network));
    let dkg_script = dkg_address.script_pubkey();
    let dust_amount = dkg_script.dust_value().to_sat();

    if amount < dust_amount {
        return Err(SBTCError::AmountInsufficient(amount, dust_amount));
    }

    let outputs = [(op_return_script, 0), (dkg_script, amount)];

    for (script, amount) in outputs.clone() {
        tx_builder.add_recipient(script, amount);
    }

    let (mut partial_tx, _) = tx_builder.finish().map_err(|err| {
        SBTCError::BDKError("Could not finish the partially signed transaction", err)
    })?;

    partial_tx.unsigned_tx.output =
        reorder_outputs(partial_tx.unsigned_tx.output.into_iter(), outputs);

    Ok(partial_tx)
}

fn deposit_data(recipient: &StacksAddress, network: Network) -> Vec<u8> {
    magic_bytes(network)
        .into_iter()
        .chain(once(b'<'))
        .chain(once(recipient.version() as u8))
        .chain(recipient.hash().as_ref().to_owned())
        .collect()
}

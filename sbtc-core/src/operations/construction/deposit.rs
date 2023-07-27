use std::{
    collections::{BTreeMap, HashMap},
    iter::once,
};

use bdk::{
    blockchain::ElectrumBlockchain, database::MemoryDatabase, electrum_client::Client,
    template::P2Wpkh, SignOptions, SyncOptions, Wallet,
};
use bitcoin::{
    blockdata::{opcodes::all::OP_RETURN, script::Builder},
    psbt::PartiallySignedTransaction,
    Address as BitcoinAddress, Network, PrivateKey, Script, Transaction, TxOut,
};
use stacks_core::address::StacksAddress;

use crate::{SBTCError, SBTCResult};

fn init_blockchain() -> SBTCResult<ElectrumBlockchain> {
    let client = Client::new("ssl://blockstream.info:993")
        .map_err(|err| SBTCError::ElectrumError("Could not create Electrum client", err))?;
    let blockchain = ElectrumBlockchain::from(client);

    Ok(blockchain)
}

fn setup_wallet(private_key: PrivateKey) -> SBTCResult<Wallet<MemoryDatabase>> {
    let blockchain = init_blockchain()?;

    let wallet = Wallet::new(
        P2Wpkh(private_key),
        Some(P2Wpkh(private_key)),
        private_key.network,
        MemoryDatabase::default(),
    )
    .map_err(|err| SBTCError::BDKError("Could not open wallet", err))?;

    wallet
        .sync(&blockchain, SyncOptions::default())
        .map_err(|err| SBTCError::BDKError("Could not sync wallet", err))?;

    Ok(wallet)
}

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

fn build_op_return_script(data: &[u8]) -> Script {
    Builder::new()
        .push_opcode(OP_RETURN)
        .push_slice(data)
        .into_script()
}

fn reorder_outputs(
    outputs: impl IntoIterator<Item = TxOut>,
    order: impl IntoIterator<Item = (Script, u64)>,
) -> Vec<TxOut> {
    let indices: HashMap<(Script, u64), usize> = order
        .into_iter()
        .enumerate()
        .map(|(idx, val)| (val, idx))
        .collect();

    let outputs_ordered: BTreeMap<usize, TxOut> = outputs
        .into_iter()
        .map(|txout| {
            (
                *indices
                    .get(&(txout.script_pubkey.clone(), txout.value))
                    .unwrap_or(&usize::MAX), // Change amount
                txout,
            )
        })
        .collect();

    outputs_ordered.into_values().collect()
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

fn magic_bytes(network: Network) -> [u8; 2] {
    match network {
        Network::Bitcoin => [b'X', b'2'],
        Network::Testnet => [b'T', b'2'],
        _ => [b'i', b'd'],
    }
}

fn deposit_data(recipient: &StacksAddress, network: Network) -> Vec<u8> {
    magic_bytes(network)
        .into_iter()
        .chain(once(b'<'))
        .chain(once(recipient.version() as u8))
        .chain(recipient.hash().as_ref().to_owned())
        .collect()
}

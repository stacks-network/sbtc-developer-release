use std::collections::{BTreeMap, HashMap};

use bdk::{
    bitcoin::{
        blockdata::{opcodes::all::OP_RETURN, script::Builder},
        PrivateKey, Script, TxOut,
    },
    blockchain::ElectrumBlockchain,
    database::MemoryDatabase,
    electrum_client::Client,
    template::P2Wpkh,
    SyncOptions, Wallet,
};

use crate::{SBTCError, SBTCResult};

/// Initializes the electrum blockchain client
pub(crate) fn init_blockchain() -> SBTCResult<ElectrumBlockchain> {
    let client = Client::new("ssl://blockstream.info:993")
        .map_err(|err| SBTCError::ElectrumError("Could not create Electrum client", err))?;
    let blockchain = ElectrumBlockchain::from(client);

    Ok(blockchain)
}

/// Set up an electrum wallet for sBTC operations
pub(crate) fn setup_wallet(private_key: PrivateKey) -> SBTCResult<Wallet<MemoryDatabase>> {
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

/// Builds an OP_RETURN script from the provided data
pub(crate) fn build_op_return_script(data: &[u8]) -> Script {
    Builder::new()
        .push_opcode(OP_RETURN)
        .push_slice(data)
        .into_script()
}

/// Reorders outputs according to the provided order
pub(crate) fn reorder_outputs(
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

use std::collections::{BTreeMap, HashMap};

use bdk::{
    bitcoin::{
        blockdata::{opcodes::all::OP_RETURN, script::Builder},
        Network, PrivateKey, Script, TxOut,
    },
    blockchain::ElectrumBlockchain,
    database::MemoryDatabase,
    electrum_client::Client,
    template::P2Wpkh,
    SyncOptions, Wallet,
};

use serde::Serialize;

pub fn init_blockchain() -> anyhow::Result<ElectrumBlockchain> {
    let client = Client::new("ssl://blockstream.info:993")?;
    let blockchain = ElectrumBlockchain::from(client);
    Ok(blockchain)
}

pub fn setup_wallet(private_key: PrivateKey) -> anyhow::Result<Wallet<MemoryDatabase>> {
    let blockchain = init_blockchain()?;
    let wallet = Wallet::new(
        P2Wpkh(private_key),
        Some(P2Wpkh(private_key)),
        private_key.network,
        MemoryDatabase::default(),
    )?;

    wallet.sync(&blockchain, SyncOptions::default())?;

    Ok(wallet)
}

pub fn build_op_return_script(data: &[u8]) -> Script {
    Builder::new()
        .push_opcode(OP_RETURN)
        .push_slice(data)
        .into_script()
}

pub fn reorder_outputs(
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

pub fn magic_bytes(network: &Network) -> [u8; 2] {
    match network {
        Network::Bitcoin => [b'X', b'2'],
        Network::Testnet => [b'T', b'2'],
        _ => [b'i', b'd'],
    }
}

#[derive(Serialize)]
pub struct TransactionData {
    pub tx_id: String,
    pub tx_hex: String,
}

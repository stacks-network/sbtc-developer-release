use std::collections::{BTreeMap, HashMap};

use bdk::bitcoin::{
	blockdata::{opcodes::all::OP_RETURN, script::Builder},
	Network, Script, TxOut,
};
use serde::Serialize;

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

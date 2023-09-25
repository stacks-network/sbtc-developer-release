//! Utilities for sBTC OP_RETURN transactions

use std::collections::{BTreeMap, HashMap};

use bdk::bitcoin::{
	blockdata::{opcodes::all::OP_RETURN, script::Builder},
	Script, TxOut,
};

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

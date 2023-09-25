//! Utils for operation construction
use std::{iter::once, num::TryFromIntError};

use bdk::bitcoin::{
	blockdata::{
		opcodes::all::{OP_CHECKSIG, OP_DROP, OP_RETURN},
		script::Builder,
	},
	schnorr::UntweakedPublicKey,
	secp256k1::Secp256k1,
	util::taproot::{
		LeafVersion, TaprootBuilder, TaprootBuilderError, TaprootSpendInfo,
	},
	Address as BitcoinAddress, Network, OutPoint, PackedLockTime, Script,
	Sequence, Transaction, TxIn, TxOut, Witness, XOnlyPublicKey,
};
use thiserror::Error;

#[derive(Error, Debug)]
/// Commit reveal error
pub enum CommitRevealError {
	#[error("Signature is invalid: {0}")]
	/// Invalid recovery ID
	InvalidRecoveryId(TryFromIntError),
	#[error("Control block could not be built from script")]
	/// No control block
	NoControlBlock,
	#[error("Could not build taproot spend info: {0}: {1}")]
	/// Taproot error
	InvalidTaproot(&'static str, TaprootBuilderError),
}

/// Commit reveal result
pub type CommitRevealResult<T> = Result<T, CommitRevealError>;

fn internal_key() -> UntweakedPublicKey {
	// Copied from BIP-0341 at https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki#constructing-and-spending-taproot-outputs
	// The BIP recommends a point
	// lift_x(0x0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0).
	// This hex string is copied from the lift_x argument with the first byte
	// stripped.
	let internal_key_vec = hex::decode(
		"50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0",
	)
	.unwrap();

	XOnlyPublicKey::from_slice(&internal_key_vec)
		.expect("Could not build internal key")
}

fn reveal_op_return_script(stacks_magic_bytes: &[u8; 2]) -> Script {
	let op_return_bytes: Vec<u8> = stacks_magic_bytes
		.iter()
		.cloned()
		.chain(once(b'w'))
		.collect();

	Builder::new()
		.push_opcode(OP_RETURN)
		.push_slice(&op_return_bytes)
		.into_script()
}

fn reclaim_script(reclaim_key: &XOnlyPublicKey) -> Script {
	Builder::new()
		.push_x_only_key(reclaim_key)
		.push_opcode(OP_CHECKSIG)
		.into_script()
}

fn op_drop_script(data: &[u8], revealer_key: &XOnlyPublicKey) -> Script {
	Builder::new()
		.push_slice(data)
		.push_opcode(OP_DROP)
		.push_x_only_key(revealer_key)
		.push_opcode(OP_CHECKSIG)
		.into_script()
}

fn address_from_taproot_spend_info(
	spend_info: TaprootSpendInfo,
) -> BitcoinAddress {
	let secp = Secp256k1::new(); // Impure call

	BitcoinAddress::p2tr(
		&secp,
		spend_info.internal_key(),
		spend_info.merkle_root(),
		Network::Testnet, // TODO: Make sure to make this configurable
	)
}

fn taproot_spend_info(
	data: &[u8],
	revealer_key: &XOnlyPublicKey,
	reclaim_key: &XOnlyPublicKey,
) -> CommitRevealResult<TaprootSpendInfo> {
	let reveal_script = op_drop_script(data, revealer_key);
	let reclaim_script = reclaim_script(reclaim_key);

	let secp = Secp256k1::new(); // Impure call
	let internal_key = internal_key();

	Ok(TaprootBuilder::new()
        .add_leaf(1, reveal_script)
        .map_err(|err| CommitRevealError::InvalidTaproot("Invalid reveal script", err))?
        .add_leaf(1, reclaim_script)
        .map_err(|err| CommitRevealError::InvalidTaproot("Invalid reclaim script", err))?
        .finalize(&secp, internal_key)
        // TODO: Confirm that this is infallible
        .expect("Taproot builder should be able to finalize after adding the internal key"))
}

/// Constructs a deposit address for the commit
pub fn commit(
	data: &[u8],
	revealer_key: &XOnlyPublicKey,
	reclaim_key: &XOnlyPublicKey,
) -> CommitRevealResult<BitcoinAddress> {
	let spend_info = taproot_spend_info(data, revealer_key, reclaim_key)?;
	Ok(address_from_taproot_spend_info(spend_info))
}

/// Data for the construction of the reveal transaction
pub struct RevealInputs<'r> {
	/// Commit output
	pub commit_output: OutPoint,
	/// Magic bytes
	pub stacks_magic_bytes: &'r [u8; 2],
	/// Revealer key
	pub revealer_key: &'r XOnlyPublicKey,
	/// Reclaim key
	pub reclaim_key: &'r XOnlyPublicKey,
}

/// Constructs a transaction that reveals the commit data
pub fn reveal(
	data: &[u8],
	RevealInputs {
		commit_output,
		stacks_magic_bytes,
		revealer_key,
		reclaim_key,
	}: RevealInputs,
) -> CommitRevealResult<Transaction> {
	let spend_info = taproot_spend_info(data, revealer_key, reclaim_key)?;

	let script = op_drop_script(data, revealer_key);
	let control_block = spend_info
		.control_block(&(script.clone(), LeafVersion::TapScript))
		.ok_or(CommitRevealError::NoControlBlock)?;

	let mut witness = Witness::new();
	witness.push(script);
	witness.push(control_block.serialize());

	let tx = Transaction {
		version: 2,
		lock_time: PackedLockTime::ZERO,
		input: vec![TxIn {
			previous_output: commit_output,
			script_sig: Script::new(),
			sequence: Sequence::MAX,
			witness,
		}],
		output: vec![TxOut {
			value: 0,
			script_pubkey: reveal_op_return_script(stacks_magic_bytes),
		}],
	};

	Ok(tx)
}

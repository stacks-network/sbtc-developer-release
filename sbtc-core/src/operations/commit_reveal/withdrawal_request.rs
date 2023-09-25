//! Primitives for sBTC commit reveal withdrawal request transactions
use std::io;

use bdk::bitcoin::{
	secp256k1::ecdsa::RecoverableSignature, Address as BitcoinAddress, Amount,
	Transaction, TxOut, XOnlyPublicKey,
};
use stacks_core::codec::Codec;

use crate::operations::{
	commit_reveal::utils::{commit, reveal, CommitRevealResult, RevealInputs},
	Opcode,
};

/// Data to construct a commit reveal withdrawal transaction
pub struct WithdrawalData {
	/// Amount to withdraw
	pub amount: Amount,
	/// Signature of the transaction
	pub signature: RecoverableSignature,
	/// How much to send for the reveal fee
	pub reveal_fee: Amount,
}

impl Codec for WithdrawalData {
	fn codec_serialize<W: io::Write>(&self, dest: &mut W) -> io::Result<()> {
		Codec::codec_serialize(&Opcode::WithdrawalRequest, dest)?;
		self.amount.codec_serialize(dest)?;
		self.signature.codec_serialize(dest)?;
		self.reveal_fee.codec_serialize(dest)
	}

	fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
	where
		Self: Sized,
	{
		let opcode = Opcode::codec_deserialize(data)
			.map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

		if !matches!(opcode, Opcode::WithdrawalRequest) {
			return Err(io::Error::new(
				io::ErrorKind::InvalidData,
				"Invalid opcode, expected withdrawal request",
			));
		}

		let amount = Amount::codec_deserialize(data)?;
		let signature = RecoverableSignature::codec_deserialize(data)?;
		let reveal_fee = Amount::codec_deserialize(data)?;

		Ok(Self {
			amount,
			signature,
			reveal_fee,
		})
	}
}

/// Constructs a withdrawal payment address
pub fn withdrawal_request_commit_address(
	withdrawal_data: WithdrawalData,
	revealer_key: &XOnlyPublicKey,
	reclaim_key: &XOnlyPublicKey,
) -> CommitRevealResult<BitcoinAddress> {
	commit(
		&withdrawal_data.serialize_to_vec(),
		revealer_key,
		reclaim_key,
	)
}

/// Constructs a transaction that reveals the withdrawal payment address
pub fn withdrawal_request_reveal_unsigned_tx(
	withdrawal_data: WithdrawalData,
	reveal_inputs: RevealInputs,
	fulfillment_fee: Amount,
	commit_amount: Amount,
	peg_wallet_address: BitcoinAddress,
	recipient_wallet_address: BitcoinAddress,
) -> CommitRevealResult<Transaction> {
	let mut tx = reveal(&withdrawal_data.serialize_to_vec(), reveal_inputs)?;

	tx.output.push(TxOut {
		value: (commit_amount - withdrawal_data.reveal_fee - fulfillment_fee)
			.to_sat(),
		script_pubkey: recipient_wallet_address.script_pubkey(),
	});
	tx.output.push(TxOut {
		value: fulfillment_fee.to_sat(),
		script_pubkey: peg_wallet_address.script_pubkey(),
	});

	Ok(tx)
}

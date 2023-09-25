//! Primitives for sBTC commit reveal deposit transactions
use std::io;

use bdk::bitcoin::{
	Address as BitcoinAddress, Amount, Transaction, TxOut, XOnlyPublicKey,
};
use stacks_core::{codec::Codec, utils::PrincipalData};

use crate::operations::{
	commit_reveal::utils::{commit, reveal, CommitRevealResult, RevealInputs},
	Opcode,
};

/// Data to construct a commit reveal deposit transaction
pub struct DepositData {
	/// Address or contract to deposit to
	pub principal: PrincipalData,
	/// How much to send for the reveal fee
	pub reveal_fee: Amount,
}

impl Codec for DepositData {
	fn codec_serialize<W: io::Write>(&self, dest: &mut W) -> io::Result<()> {
		Codec::codec_serialize(&Opcode::Deposit, dest)?;
		self.principal.codec_serialize(dest)?;
		self.reveal_fee.codec_serialize(dest)?;

		todo!()
	}

	fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
	where
		Self: Sized,
	{
		let opcode = Opcode::codec_deserialize(data)
			.map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

		if !matches!(opcode, Opcode::Deposit) {
			return Err(io::Error::new(
				io::ErrorKind::InvalidData,
				"Invalid opcode, expected deposit",
			));
		}

		let principal = PrincipalData::codec_deserialize(data)?;
		let reveal_fee = Amount::codec_deserialize(data)?;

		Ok(Self {
			principal,
			reveal_fee,
		})
	}
}

/// Constructs a deposit payment address
pub fn deposit_commit_address(
	deposit_data: DepositData,
	revealer_key: &XOnlyPublicKey,
	reclaim_key: &XOnlyPublicKey,
) -> CommitRevealResult<BitcoinAddress> {
	commit(&deposit_data.serialize_to_vec(), revealer_key, reclaim_key)
}

/// Constructs a transaction that reveals the deposit payment address
pub fn deposit_reveal_unsigned_tx(
	deposit_data: DepositData,
	reveal_inputs: RevealInputs,
	commit_amount: Amount,
	peg_wallet_address: BitcoinAddress,
) -> CommitRevealResult<Transaction> {
	let mut tx = reveal(&deposit_data.serialize_to_vec(), reveal_inputs)?;

	tx.output.push(TxOut {
		value: (commit_amount - deposit_data.reveal_fee).to_sat(),
		script_pubkey: peg_wallet_address.script_pubkey(),
	});

	Ok(tx)
}

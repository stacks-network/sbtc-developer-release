//! Tools for the construction and parsing of the sBTC OP_RETURN withdrawal
//! request transactions.
//!
//! Withdrawal request is a Bitcoin transaction with the output structure as
//! below:
//!
//! 1. data output
//! 2. Bitcoin address to send the BTC to
//! 3. Fulfillment fee payment to the peg wallet
//!
//! The data output should contain data in the following byte format:
//!
//! ```text
//! 0     2  3                                                                    80
//! |-----|--|---------------------------------------------------------------------|
//! magic op                       withdrawal request data
//! ```
//!
//! Where withdrawal request data should be in the following format:
//!
//! ```text
//! 3         11                                                            76    80
//! |----------|-------------------------------------------------------------|-----|
//! amount                            signature                            extra
//! bytes
use std::{collections::HashMap, io};

use bdk::{
	bitcoin::{
		psbt::PartiallySignedTransaction,
		secp256k1::{ecdsa::RecoverableSignature, Message, Secp256k1},
		Address as BitcoinAddress, Amount, Network, PrivateKey, Transaction,
	},
	database::MemoryDatabase,
	SignOptions, Wallet,
};
use stacks_core::{
	codec::Codec,
	crypto::{sha256::Sha256Hasher, Hashing},
};

use crate::{
	operations::{
		magic_bytes,
		op_return::utils::{build_op_return_script, reorder_outputs},
		utils::setup_wallet,
		Opcode,
	},
	SBTCError, SBTCResult,
};

#[derive(PartialEq, Eq, Debug)]
/// Data for the sBTC OP_RETURN withdrawal request transaction output
pub struct WithdrawalRequestOutputData {
	/// Network to be used for the transaction
	pub network: Network,
	/// Amount to withdraw
	pub amount: Amount,
	/// Signature of the withdrawal request amount and recipient address
	pub signature: RecoverableSignature,
}

impl Codec for WithdrawalRequestOutputData {
	fn codec_serialize<W: io::Write>(&self, dest: &mut W) -> io::Result<()> {
		dest.write_all(&magic_bytes(self.network))?;
		dest.write_all(&[Opcode::WithdrawalRequest as u8])?;
		self.amount.codec_serialize(dest)?;
		self.signature.codec_serialize(dest)
	}

	fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
	where
		Self: Sized,
	{
		let mut magic_bytes_buffer = [0; 2];
		data.read_exact(&mut magic_bytes_buffer)?;

		let network_magic_bytes = [
			Network::Bitcoin,
			Network::Testnet,
			Network::Signet,
			Network::Regtest,
		]
		.into_iter()
		.map(|network| (magic_bytes(network), network))
		.collect::<HashMap<[u8; 2], Network>>();

		let network = network_magic_bytes
			.get(&magic_bytes_buffer)
			.cloned()
			.ok_or(io::Error::new(
				io::ErrorKind::InvalidData,
				format!("Unknown magic bytes: {:?}", magic_bytes_buffer),
			))?;

		let opcode = Opcode::codec_deserialize(data)
			.map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

		if !matches!(opcode, Opcode::WithdrawalRequest) {
			return Err(io::Error::new(
				io::ErrorKind::InvalidData,
				format!(
					"Invalid opcode, expected withdrawal request: {:?}",
					opcode
				),
			));
		}

		let amount = Amount::codec_deserialize(data)?;
		let signature = RecoverableSignature::codec_deserialize(data)
			.map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

		Ok(Self {
			network,
			amount,
			signature,
		})
	}
}

fn sign_amount_and_recipient(
	recipient: &BitcoinAddress,
	amount: Amount,
	sender_private_key: &PrivateKey,
) -> RecoverableSignature {
	let mut msg = amount.to_sat().to_be_bytes().to_vec();
	msg.extend_from_slice(recipient.script_pubkey().as_bytes());

	let msg_hash = Sha256Hasher::hash(&msg);
	let msg_ecdsa = Message::from_slice(msg_hash.as_ref()).unwrap();

	Secp256k1::new()
		.sign_ecdsa_recoverable(&msg_ecdsa, &sender_private_key.inner)
}

fn withdrawal_psbt(
	wallet: &Wallet<MemoryDatabase>,
	sender_private_key: &PrivateKey,
	recipient: &BitcoinAddress,
	dkg_address: &BitcoinAddress,
	amount: Amount,
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

	let signature =
		sign_amount_and_recipient(recipient, amount, sender_private_key);
	let op_return_script = build_op_return_script(
		&WithdrawalRequestOutputData {
			network,
			amount,
			signature,
		}
		.serialize_to_vec(),
	);

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

	partial_tx.unsigned_tx.output =
		reorder_outputs(partial_tx.unsigned_tx.output, outputs);

	Ok(partial_tx)
}

/// Construct a BTC transaction containing the provided sBTC withdrawal data
pub fn build_withdrawal_tx(
	withdrawer_bitcoin_private_key: PrivateKey,
	withdrawer_stacks_private_key: PrivateKey,
	receiver_address: BitcoinAddress,
	amount: Amount,
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
		.map_err(|err| {
			SBTCError::BDKError("Could not sign withdrawal transaction", err)
		})?;

	Ok(psbt.extract_tx())
}

//! Tools for the construction and parsing of the sBTC OP_RETURN withdrawal
//! request transactions.
//!
//!
//! Withdrawal request is a Bitcoin transaction with three counterparties:
//!
//! 1. broadcaster - broadcasts the transaction
//! 2. drawee - burns sBTC
//! 3. payee - gets BTC
//!
//! From the perspective of private keys the broadcaster and payee can be the
//! same party, while drawee is generally a different private key.
//!
//! Its output structure is as below:
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
//! 0          8                                                                  72
//! |----------|-------------------------------------------------------------------|
//! amount                                signature
//! ```
//!
//! The signature is a recoverable ECDSA signature produced by signing the
//! following message:
//!
//! ```text
//! 0   1            N   N + 1                                             M + N + 1
//! |---|------------|---|---------------------------------------------------------|
//! ^     prefix     ^                     message data
//! |                |
//! prefix length    message data length
//! ```
//!
//! This prefix is by convention always [`STACKS_SIGNATURE_PREFIX`]. Message
//! data is a concatenation of the amount (BE bytes) and the pubkey script of
//! the recipient Bitcoin address.
//!
//! ```text
//! 0                8                                                             N
//! |----------------|-------------------------------------------------------------|
//! amount                             pubkey script
//! ```
//!
//! It is also by convention that we always produce a P2PKH Stacks address from
//! the recovered public key.
use std::{collections::HashMap, io, iter};

use bdk::{
    bitcoin::{
        blockdata::{opcodes::all::OP_RETURN, script::Instruction},
        psbt::PartiallySignedTransaction,
        secp256k1::{ecdsa::RecoverableSignature, Message, Secp256k1},
        Address as BitcoinAddress, Amount, Network, PrivateKey, Transaction,
    },
    database::MemoryDatabase,
    SignOptions, Wallet,
};
use stacks_core::{
	address::{AddressVersion as StacksAddressVersion, StacksAddress},
	codec::Codec,
	crypto::{
		sha256::Sha256Hasher, Hashing, PrivateKey as StacksPrivateKey,
		PublicKey as StacksPublicKey,
	},
};
use thiserror::Error;

use crate::{
	operations::{
		magic_bytes,
		op_return::utils::{build_op_return_script, reorder_outputs},
		Opcode,
	},
	SBTCError, SBTCResult,
};

#[derive(Error, Debug)]
/// Errors occuring during the parsing of the withdrawal request
pub enum WithdrawalParseError {
    /// Missing expected output
    #[error("Missing an expected output")]
    InvalidOutputs,

    /// Doesn't contain an OP_RETURN with the right opcode
    #[error("Not an sBTC operation")]
    NotSbtcOp,

    /// A recipient address error
    #[error("Could not get recipient address from output")]
    InvalidRecipientAddress,
}

/// Amount and a recipient for a withdrawal request
pub struct WithdrawalRequest {
    recipient_address: BitcoinAddress,
    amount: Amount,
    fulfillment_amount: Amount,
    peg_wallet: BitcoinAddress,
}

impl WithdrawalRequest {
    /// Parse a withdrawal request from a transaction
    pub fn parse(network: Network, tx: Transaction) -> Result<Self, WithdrawalParseError> {
        let mut output_iter = tx.output.into_iter();

        let data_output = output_iter
            .next()
            .ok_or(WithdrawalParseError::InvalidOutputs)?;

        let mut instructions_iter = data_output.script_pubkey.instructions();

        let Some(Ok(Instruction::Op(OP_RETURN))) = instructions_iter.next() else {
            return Err(WithdrawalParseError::NotSbtcOp);
        };

        let Some(Ok(Instruction::PushBytes(mut data))) = instructions_iter.next() else {
            return Err(WithdrawalParseError::NotSbtcOp);
        };

        let withdrawal_data = WithdrawalRequestOutputData::codec_deserialize(&mut data)
            .map_err(|_| WithdrawalParseError::NotSbtcOp)?;

        let recipient_pubkey_output = output_iter
            .next()
            .ok_or(WithdrawalParseError::InvalidOutputs)?;

        let recipient_address =
            BitcoinAddress::from_script(&recipient_pubkey_output.script_pubkey, network)
                .map_err(|_| WithdrawalParseError::InvalidRecipientAddress)?;

        let fulfillment_fee_output = output_iter
            .next()
            .ok_or(WithdrawalParseError::InvalidOutputs)?;

        let peg_wallet =
            BitcoinAddress::from_script(&fulfillment_fee_output.script_pubkey, network)
                .map_err(|_| WithdrawalParseError::InvalidRecipientAddress)?;

        let fulfillment_amount = Amount::from_sat(fulfillment_fee_output.value);

        Ok(Self {
            recipient_address,
            amount: withdrawal_data.amount,
            fulfillment_amount,
            peg_wallet,
        })
    }
}

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

/// Construct a withdrawal request transaction
pub fn build_withdrawal_tx(
	wallet: &Wallet<impl BatchDatabase>,
	broadcaster_bitcoin_private_key: BitcoinPrivateKey,
	drawee_stacks_private_key: StacksPrivateKey,
	payee_bitcoin_address: BitcoinAddress,
	peg_wallet_bitcoin_address: BitcoinAddress,
	amount: u64,
	fulfillment_fee: u64,
) -> SBTCResult<Transaction> {
	let mut psbt = create_psbt(
		wallet,
		&drawee_stacks_private_key,
		&payee_bitcoin_address,
		&peg_wallet_bitcoin_address,
		amount,
		fulfillment_fee,
		broadcaster_bitcoin_private_key.network,
	)?;

	wallet
		.sign(&mut psbt, SignOptions::default())
		.map_err(|err| {
			SBTCError::BDKError("Could not sign withdrawal transaction", err)
		})?;

	Ok(psbt.extract_tx())
}

/// Construct a withdrawal request partially signed transaction
pub fn create_psbt<D: BatchDatabase>(
	wallet: &Wallet<D>,
	drawee_stacks_private_key: &StacksPrivateKey,
	payee_bitcoin_address: &BitcoinAddress,
	peg_wallet_bitcoin_address: &BitcoinAddress,
	amount: u64,
	fulfillment_amount: u64,
	network: BitcoinNetwork,
) -> SBTCResult<PartiallySignedTransaction> {
	let outputs = create_outputs(
		drawee_stacks_private_key,
		payee_bitcoin_address,
		peg_wallet_bitcoin_address,
		amount,
		fulfillment_amount,
		network,
	)?;

	let mut tx_builder = wallet.build_tx();

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

/// Generates the outputs for the withdrawal request transaction
pub fn create_outputs(
	drawee_stacks_private_key: &StacksPrivateKey,
	payee_bitcoin_address: &BitcoinAddress,
	peg_wallet_bitcoin_address: &BitcoinAddress,
	amount: u64,
	fulfillment_amount: u64,
	network: BitcoinNetwork,
) -> SBTCResult<[(Script, u64); 3]> {
	let recipient_script = payee_bitcoin_address.script_pubkey();
	let dkg_wallet_script = peg_wallet_bitcoin_address.script_pubkey();

	// Check that we have enough to cover dust
	let recipient_dust_amount = recipient_script.dust_value().to_sat();
	let dkg_wallet_dust_amount = dkg_wallet_script.dust_value().to_sat();

	if fulfillment_amount < dkg_wallet_dust_amount {
		return Err(SBTCError::AmountInsufficient(
			fulfillment_amount,
			dkg_wallet_dust_amount,
		));
	}

	let op_return_script = build_op_return_script(
		&WithdrawalRequestDataOutputData::new(
			payee_bitcoin_address,
			drawee_stacks_private_key,
			amount,
			network,
		)
		.serialize_to_vec(),
	);

	let outputs = [
		(op_return_script, 0),
		(recipient_script, recipient_dust_amount),
		(dkg_wallet_script, fulfillment_amount),
	];

	Ok(outputs)
}

#[derive(PartialEq, Eq, Debug)]
/// Data for the sBTC OP_RETURN withdrawal request transaction output
pub struct WithdrawalRequestDataOutputData {
	/// BitcoinNetwork to be used for the transaction
	network: BitcoinNetwork,
	/// Amount to withdraw
	amount: u64,
	/// Signature of the withdrawal request amount and recipient address
	signature: RecoverableSignature,
}

impl WithdrawalRequestDataOutputData {
	/// Creates a new withdrawal request output data with signature
	pub fn new(
		payee_bitcoin_address: &BitcoinAddress,
		drawee_stacks_private_key: &StacksPrivateKey,
		amount: u64,
		network: BitcoinNetwork,
	) -> Self {
		let signature = create_signature(
			drawee_stacks_private_key,
			payee_bitcoin_address,
			amount,
		);

		Self {
			network,
			amount,
			signature,
		}
	}

	/// Returns the withdrawal request network
	pub fn network(&self) -> BitcoinNetwork {
		self.network
	}

	/// Returns the withdrawal request amount
	pub fn amount(&self) -> u64 {
		self.amount
	}

	/// Returns the withdrawal request signature
	pub fn signature(&self) -> RecoverableSignature {
		self.signature
	}
}

impl Codec for WithdrawalRequestDataOutputData {
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
			BitcoinNetwork::Bitcoin,
			BitcoinNetwork::Testnet,
			BitcoinNetwork::Signet,
			BitcoinNetwork::Regtest,
		]
		.into_iter()
		.map(|network| (magic_bytes(network), network))
		.collect::<HashMap<[u8; 2], BitcoinNetwork>>();

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

		let amount = u64::codec_deserialize(data)?;
		let signature = RecoverableSignature::codec_deserialize(data)
			.map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

		Ok(Self {
			network,
			amount,
			signature,
		})
	}
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
        .map_err(|err| SBTCError::BDKError("Could not sign withdrawal transaction", err))?;

    Ok(psbt.extract_tx())
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

	Secp256k1::new().sign_ecdsa_recoverable(&msg, drawee_stacks_private_key)
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

    Secp256k1::new().sign_ecdsa_recoverable(&msg_ecdsa, &sender_private_key.inner)
}

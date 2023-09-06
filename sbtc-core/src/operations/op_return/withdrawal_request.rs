/*!
Tools for the construction and parsing of the sBTC OP_RETURN withdrawal request
transactions.

Withdrawal request is a Bitcoin transaction with the output structure as below:

1. data output
2. Bitcoin address to send the BTC to
3. Fulfillment fee payment to the peg wallet

The data output should contain data in the following byte format:

```text
0     2  3                                                                    80
|-----|--|---------------------------------------------------------------------|
 magic op                       withdrawal request data
```

Where withdrawal request data should be in the following format:

```text
3         11                                                            76    80
|----------|-------------------------------------------------------------|-----|
   amount                            signature                            extra
                                                                          bytes
*/
use std::{collections::HashMap, io, iter};

use bdk::{
    bitcoin::{
        blockdata::{opcodes::all::OP_RETURN, script::Instruction},
        psbt::PartiallySignedTransaction,
        secp256k1::{self, ecdsa::RecoverableSignature, Message, Secp256k1, SecretKey},
        Address as BitcoinAddress, Network as BitcoinNetwork, PrivateKey, Transaction,
    },
    database::MemoryDatabase,
    SignOptions, Wallet,
};
use stacks_core::{
    address::StacksAddress,
    codec::Codec,
    crypto::{sha256::Sha256Hasher, Hashing},
};
use thiserror::Error;

use crate::{
    operations::{
        magic_bytes,
        op_return::utils::{build_op_return_script, reorder_outputs},
        utils::setup_wallet,
        Opcode,
    },
    SBTCError, SBTCResult,
};

/// Signature prefix used by convention
const STACKS_SIGNATURE_PREFIX: &[u8] = b"Stacks Signed Message:\n";

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
    /// Where to send the withdrawn BTC
    pub recipient: BitcoinAddress,
    /// Where to burn the sBTC from
    pub source: StacksAddress,
    /// How much to withdraw
    pub amount: u64,
    /// How much to pay the peg wallet for the fulfillment
    pub fulfillment_amount: u64,
    /// The address of the peg wallet
    pub peg_wallet: BitcoinAddress,
}

impl WithdrawalRequest {
    /// Parse a withdrawal request from a transaction
    pub fn parse(network: BitcoinNetwork, tx: Transaction) -> Result<Self, WithdrawalParseError> {
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

        let source_public_key = withdrawal_data.source_public_key(&recipient_address);
        let source =
            todo!("Figure out how to know the address type to compute it from the public key");

        let fulfillment_fee_output = output_iter
            .next()
            .ok_or(WithdrawalParseError::InvalidOutputs)?;

        let peg_wallet =
            BitcoinAddress::from_script(&fulfillment_fee_output.script_pubkey, network)
                .map_err(|_| WithdrawalParseError::InvalidRecipientAddress)?;

        Ok(Self {
            recipient: recipient_address,
            source,
            amount: withdrawal_data.amount(),
            fulfillment_amount: fulfillment_fee_output.value,
            peg_wallet,
        })
    }
}

#[derive(PartialEq, Eq, Debug)]
/// Data for the sBTC OP_RETURN withdrawal request transaction output
pub struct WithdrawalRequestOutputData {
    /// BitcoinNetwork to be used for the transaction
    network: BitcoinNetwork,
    /// Amount to withdraw
    amount: u64,
    /// Signature of the withdrawal request amount and recipient address
    signature: RecoverableSignature,
}

impl WithdrawalRequestOutputData {
    /// Creates a new withdrawal request output data with signature
    pub fn new(
        network: BitcoinNetwork,
        amount: u64,
        recipient: &BitcoinAddress,
        sender_secret_key: &SecretKey,
    ) -> Self {
        let signing_msg = create_withdrawal_request_signing_message(amount, recipient);
        let signature = Secp256k1::new().sign_ecdsa_recoverable(&signing_msg, sender_secret_key);

        Self {
            network,
            amount,
            signature,
        }
    }

    /// Computes withdrawal request source public key
    pub fn source_public_key(
        &self,
        recipient: &BitcoinAddress,
    ) -> SBTCResult<secp256k1::PublicKey> {
        let signing_msg = create_withdrawal_request_signing_message(self.amount, recipient);

        Secp256k1::new()
            .recover_ecdsa(&signing_msg, &self.signature)
            .map_err(|err| SBTCError::SECPError("Could not recover public key from signature", err))
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
                format!("Invalid opcode, expected withdrawal request: {:?}", opcode),
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

/// Computes a public key of the sender from the signature
pub fn compute_withdrawal_request_sender_public_key(
    amount: u64,
    recipient: &BitcoinAddress,
    signature: &RecoverableSignature,
) -> SBTCResult<secp256k1::PublicKey> {
    let signing_msg = create_withdrawal_request_signing_message(amount, recipient);

    Secp256k1::new()
        .recover_ecdsa(&signing_msg, &signature)
        .map_err(|err| SBTCError::SECPError("Could not recover public key from signature", err))
}

/// Creates the SECP signing message for the withdrawal request
pub fn create_withdrawal_request_signing_message(
    amount: u64,
    recipient: &BitcoinAddress,
) -> Message {
    let signing_data: Vec<u8> = iter::empty()
        .chain(amount.serialize_to_vec())
        .chain(recipient.script_pubkey().as_bytes().to_vec())
        .collect();

    create_signing_message(signing_data)
}

/**
Creates the SECP signing message. It prepends the data with the
*Stacks signature prefix* that is used by convention.
*/
pub fn create_signing_message(data: impl AsRef<[u8]>) -> Message {
    // Both the Stacks prefix and the data need to be preceded by their length
    let msg_content: Vec<u8> = iter::empty()
        .chain(iter::once(STACKS_SIGNATURE_PREFIX.len() as u8))
        .chain(STACKS_SIGNATURE_PREFIX.iter().copied())
        .chain(iter::once(data.as_ref().len() as u8))
        .chain(data.as_ref().iter().copied())
        .collect();

    Message::from_slice(Sha256Hasher::new(msg_content).as_ref())
        .expect("Could not create secp message")
}

/// Construct a BTC transaction containing the provided sBTC withdrawal data
pub fn build_withdrawal_tx(
    withdrawer_bitcoin_private_key: PrivateKey,
    withdrawer_stacks_private_key: PrivateKey,
    receiver_address: BitcoinAddress,
    amount: u64,
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
    amount: u64,
    fulfillment_fee: u64,
    network: BitcoinNetwork,
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

    let signature = sign_amount_and_recipient(recipient, amount, sender_private_key);
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

    partial_tx.unsigned_tx.output = reorder_outputs(partial_tx.unsigned_tx.output, outputs);

    Ok(partial_tx)
}

fn sign_amount_and_recipient(
    recipient: &BitcoinAddress,
    amount: u64,
    sender_private_key: &PrivateKey,
) -> RecoverableSignature {
    let mut msg = amount.serialize_to_vec();
    msg.extend_from_slice(recipient.script_pubkey().as_bytes());

    let msg_hash = Sha256Hasher::hash(&msg);
    let msg_ecdsa = Message::from_slice(msg_hash.as_ref()).unwrap();

    Secp256k1::new().sign_ecdsa_recoverable(&msg_ecdsa, &sender_private_key.inner)
}

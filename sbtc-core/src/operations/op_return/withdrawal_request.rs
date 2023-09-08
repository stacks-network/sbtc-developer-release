/*!
Tools for the construction and parsing of the sBTC OP_RETURN withdrawal request
transactions.


Withdrawal request is a Bitcoin transaction with three counterparties:

1. broadcaster - broadcasts the transaction
2. drawee - burns sBTC
3. payee - gets BTC

From the perspective of private keys the broadcaster and payee can be the same
party, while drawee is generally a different private key.

Its output structure is as below:

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
0          8                                                                  72
|----------|-------------------------------------------------------------------|
   amount                                signature
```

The signature is a recoverable ECDSA signature produced by signing the following
message:

```text
0   1            N   N + 1                                             M + N + 1
|---|------------|---|---------------------------------------------------------|
  ^     prefix     ^                     message data
  |                |
 prefix length    message data length
```

This prefix is by convention always [`STACKS_SIGNATURE_PREFIX`]. Message data is
a concatenation of the amount (BE bytes) and the pubkey script of the recipient
Bitcoin address.

```text
0                8                                                             N
|----------------|-------------------------------------------------------------|
      amount                             pubkey script
```

It is also by convention that we always produce a P2PKH Stacks address from the
recovered public key.
*/
use std::{collections::HashMap, io, iter};

use bdk::{
    bitcoin::{
        blockdata::{opcodes::all::OP_RETURN, script::Instruction},
        psbt::PartiallySignedTransaction,
        secp256k1::{ecdsa::RecoverableSignature, Message, Secp256k1},
        Address as BitcoinAddress, Network as BitcoinNetwork, PrivateKey as BitcoinPrivateKey,
        Script, Transaction,
    },
    database::BatchDatabase,
    SignOptions, Wallet,
};
use stacks_core::{
    address::{AddressVersion as StacksAddressVersion, StacksAddress},
    codec::Codec,
    crypto::{
        sha256::Sha256Hasher, Hashing, PrivateKey as StacksPrivateKey, PublicKey as StacksPublicKey,
    },
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

/// Signature prefix used by convention
pub const STACKS_SIGNATURE_PREFIX: &[u8] = b"Stacks Signed Message:\n";

/// Tries to parse a Bitcoin transation into a withdrawal request
pub fn try_parse_withdrawal_request(
    network: BitcoinNetwork,
    tx: Transaction,
) -> SBTCResult<WithdrawalRequestData> {
    let mut output_iter = tx.output.into_iter();

    let data_output = output_iter.next().ok_or(SBTCError::NotSBTCOperation)?;

    let mut instructions_iter = data_output.script_pubkey.instructions();

    let Some(Ok(Instruction::Op(OP_RETURN))) = instructions_iter.next() else {
        return Err(SBTCError::NotSBTCOperation);
    };

    let Some(Ok(Instruction::PushBytes(mut data))) = instructions_iter.next() else {
        return Err(SBTCError::NotSBTCOperation);
    };

    let withdrawal_data = WithdrawalRequestDataOutputData::codec_deserialize(&mut data)
        .map_err(|_| SBTCError::NotSBTCOperation)?;

    let recipient_pubkey_output = output_iter.next().ok_or(SBTCError::NotSBTCOperation)?;

    let recipient_address =
        BitcoinAddress::from_script(&recipient_pubkey_output.script_pubkey, network)
            .map_err(|_| SBTCError::NotSBTCOperation)?;

    let drawee_stacks_public_key = recover_signature(
        withdrawal_data.amount(),
        &recipient_address,
        &withdrawal_data.signature(),
    )?;
    let drawee_stacks_address_version = match network {
        BitcoinNetwork::Bitcoin => StacksAddressVersion::MainnetSingleSig,
        _ => StacksAddressVersion::TestnetSingleSig,
    };
    let drawee_stacks_address =
        StacksAddress::from_public_key(drawee_stacks_address_version, &drawee_stacks_public_key);

    let fulfillment_fee_output = output_iter.next().ok_or(SBTCError::NotSBTCOperation)?;

    let peg_wallet = BitcoinAddress::from_script(&fulfillment_fee_output.script_pubkey, network)
        .map_err(|_| SBTCError::NotSBTCOperation)?;

    Ok(WithdrawalRequestData {
        payee_bitcoin_address: recipient_address,
        drawee_stacks_address,
        amount: withdrawal_data.amount(),
        signature: withdrawal_data.signature(),
        fulfillment_amount: fulfillment_fee_output.value,
        peg_wallet,
    })
}

/// Withdrawal request transaction data
pub struct WithdrawalRequestData {
    /// Where to send the withdrawn BTC
    pub payee_bitcoin_address: BitcoinAddress,
    /// Where to burn the sBTC from
    pub drawee_stacks_address: StacksAddress,
    /// How much to withdraw
    pub amount: u64,
    /// How much to pay the peg wallet for the fulfillment
    pub fulfillment_amount: u64,
    /// The address of the peg wallet
    pub peg_wallet: BitcoinAddress,
    /// Signature that authenticates the withdrawal request
    pub signature: RecoverableSignature,
}

impl WithdrawalRequestData {
    /// Recovers the signature and computes the Stacks address to be burned from
    pub fn get_drawee_address(&self) -> StacksAddress {
        todo!()
    }
}

/// Construct a withdrawal request transaction
pub fn build_withdrawal_tx(
    broadcaster_bitcoin_private_key: BitcoinPrivateKey,
    drawee_stacks_private_key: StacksPrivateKey,
    payee_bitcoin_address: BitcoinAddress,
    peg_wallet_bitcoin_address: BitcoinAddress,
    amount: u64,
    fulfillment_fee: u64,
) -> SBTCResult<Transaction> {
    let wallet = setup_wallet(broadcaster_bitcoin_private_key)?;

    let mut psbt = create_psbt(
        &wallet,
        &drawee_stacks_private_key,
        &payee_bitcoin_address,
        &peg_wallet_bitcoin_address,
        amount,
        fulfillment_fee,
        broadcaster_bitcoin_private_key.network,
    )?;

    wallet
        .sign(&mut psbt, SignOptions::default())
        .map_err(|err| SBTCError::BDKError("Could not sign withdrawal transaction", err))?;

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

    partial_tx.unsigned_tx.output = reorder_outputs(partial_tx.unsigned_tx.output, outputs);

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
        let signature = create_signature(drawee_stacks_private_key, payee_bitcoin_address, amount);

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

/// Creates the signature for the withdrawal request
pub fn create_signature(
    drawee_stacks_private_key: &StacksPrivateKey,
    payee_bitcoin_address: &BitcoinAddress,
    amount: u64,
) -> RecoverableSignature {
    let msg = create_withdrawal_request_signing_message(amount, payee_bitcoin_address);

    Secp256k1::new().sign_ecdsa_recoverable(&msg, drawee_stacks_private_key)
}

/// Recovers a Stacks public key of the payee from the signature
pub fn recover_signature(
    amount: u64,
    payee_bitcoin_address: &BitcoinAddress,
    signature: &RecoverableSignature,
) -> SBTCResult<StacksPublicKey> {
    let signing_msg = create_withdrawal_request_signing_message(amount, payee_bitcoin_address);

    Secp256k1::new()
        .recover_ecdsa(&signing_msg, signature)
        .map_err(|err| SBTCError::SECPError("Could not recover public key from signature", err))
}

/// Creates the SECP signing message for the withdrawal request
pub fn create_withdrawal_request_signing_message(
    amount: u64,
    payee_bitcoin_address: &BitcoinAddress,
) -> Message {
    let signing_data: Vec<u8> = iter::empty()
        .chain(amount.serialize_to_vec())
        .chain(payee_bitcoin_address.script_pubkey().as_bytes().to_vec())
        .collect();

    create_signing_message(signing_data)
}

/**
Creates the SECP signing message. It prepends the data with the
[`STACKS_SIGNATURE_PREFIX`] that is used by convention.
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

/*!
Construction of commit reveal transactions
*/
use std::io;

use bdk::bitcoin::{
    secp256k1::ecdsa::RecoverableSignature, Address as BitcoinAddress, Amount, Transaction, TxOut,
    XOnlyPublicKey,
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
        self.amount.codec_serialize(dest)?;
        self.signature.codec_serialize(dest)?;
        self.reveal_fee.codec_serialize(dest)
    }

    fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
    where
        Self: Sized,
    {
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

/// Constructs a peg in payment address
pub fn deposit_commit(
    deposit_data: DepositData,
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> CommitRevealResult<BitcoinAddress> {
    commit(&deposit_data.serialize_to_vec(), revealer_key, reclaim_key)
}

/// Constructs a peg out payment address
pub fn withdrawal_request_commit(
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

/// Constructs a transaction that reveals the peg in payment address
pub fn deposit_reveal_unsigned(
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

/// Constructs a transaction that reveals the peg out payment address
pub fn withdrawal_request_reveal_unsigned(
    withdrawal_data: WithdrawalData,
    reveal_inputs: RevealInputs,
    fulfillment_fee: Amount,
    commit_amount: Amount,
    peg_wallet_address: BitcoinAddress,
    recipient_wallet_address: BitcoinAddress,
) -> CommitRevealResult<Transaction> {
    let mut tx = reveal(&withdrawal_data.serialize_to_vec(), reveal_inputs)?;

    tx.output.push(TxOut {
        value: (commit_amount - withdrawal_data.reveal_fee - fulfillment_fee).to_sat(),
        script_pubkey: recipient_wallet_address.script_pubkey(),
    });
    tx.output.push(TxOut {
        value: fulfillment_fee.to_sat(),
        script_pubkey: peg_wallet_address.script_pubkey(),
    });

    Ok(tx)
}

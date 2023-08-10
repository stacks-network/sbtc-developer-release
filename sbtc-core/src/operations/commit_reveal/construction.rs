/*!
Construction of commit reveal transactions
*/
use std::iter::{once, repeat};

use bdk::bitcoin::{
    secp256k1::ecdsa::RecoverableSignature, Address as BitcoinAddress, Transaction, TxOut,
    XOnlyPublicKey,
};
use stacks_core::address::StacksAddress;

use crate::operations::{
    commit_reveal::utils::{commit, reveal, CommitRevealError, CommitRevealResult, RevealInputs},
    Opcode,
};

/// Data to construct a commit reveal deposit transaction
pub struct DepositData<'p> {
    /// Address to deposit to
    pub address: &'p StacksAddress,
    /// Name of the contract to deposit to
    pub contract_name: Option<&'p str>,
    /// How much to send for the reveal fee
    pub reveal_fee: u64,
}

impl<'p> DepositData<'p> {
    /// Create deposit data
    pub fn new(
        address: &'p StacksAddress,
        contract_name: Option<&'p str>,
        reveal_fee: u64,
    ) -> Self {
        Self {
            address,
            contract_name,
            reveal_fee,
        }
    }

    /// Serializes this data according to the SIP-021 wire formats
    /// Links:
    ///  - [SIP draft](https://github.com/stacksgov/sips/blob/56b73eada5ef1b72376f4a230949297b3edcc562/sips/sip-021/sip-021-trustless-two-way-peg-for-bitcoin.md)
    ///  - [Reference implementation](https://github.com/stacks-network/stacks-blockchain/blob/next/src/chainstate/burn/operations/peg_in.rs)
    pub fn to_vec(&self) -> Vec<u8> {
        once(Opcode::Deposit as u8)
            .chain(once(self.address.version() as u8))
            .chain(self.address.hash().as_ref().iter().cloned())
            .chain(
                self.contract_name
                    .map(|contract_name| contract_name.as_bytes().to_vec())
                    .into_iter()
                    .flatten(),
            )
            .chain(repeat(0))
            .take(78)
            .chain(self.reveal_fee.to_be_bytes())
            .collect()
    }
}

/// Data to construct a commit reveal withdrawal transaction
pub struct WithdrawalData<'r> {
    /// Amount to withdraw
    pub amount: u64,
    /// Signature of the transaction
    pub signature: &'r RecoverableSignature,
    /// How much to send for the reveal fee
    pub reveal_fee: u64,
}

impl<'r> WithdrawalData<'r> {
    /// Create withdrawal data
    pub fn new(amount: u64, signature: &'r RecoverableSignature, reveal_fee: u64) -> Self {
        Self {
            amount,
            signature,
            reveal_fee,
        }
    }

    /// Serialize withdrawal data
    pub fn to_vec(&self) -> CommitRevealResult<Vec<u8>> {
        let (recovery_id, signature_bytes) = self.signature.serialize_compact();
        let recovery_id: u8 = recovery_id
            .to_i32()
            .try_into()
            .map_err(CommitRevealError::InvalidRecoveryId)?;
        let empty_memo = [0; 4];

        Ok(once(Opcode::WithdrawalRequest as u8)
            .chain(self.amount.to_be_bytes())
            .chain(once(recovery_id))
            .chain(signature_bytes)
            .chain(empty_memo)
            .chain(self.reveal_fee.to_be_bytes())
            .collect())
    }
}

/// Constructs a peg in payment address
pub fn deposit_commit<'p>(
    deposit_data: DepositData<'p>,
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> CommitRevealResult<BitcoinAddress> {
    commit(&deposit_data.to_vec(), revealer_key, reclaim_key)
}

/// Constructs a peg out payment address
pub fn withdrawal_request_commit(
    withdrawal_data: WithdrawalData,
    revealer_key: &XOnlyPublicKey,
    reclaim_key: &XOnlyPublicKey,
) -> CommitRevealResult<BitcoinAddress> {
    commit(&withdrawal_data.to_vec()?, revealer_key, reclaim_key)
}

/// Constructs a transaction that reveals the peg in payment address
pub fn deposit_reveal_unsigned<'p>(
    deposit_data: DepositData<'p>,
    reveal_inputs: RevealInputs,
    commit_amount: u64,
    peg_wallet_address: BitcoinAddress,
) -> CommitRevealResult<Transaction> {
    let mut tx = reveal(&deposit_data.to_vec(), reveal_inputs)?;

    tx.output.push(TxOut {
        value: commit_amount - deposit_data.reveal_fee,
        script_pubkey: peg_wallet_address.script_pubkey(),
    });

    Ok(tx)
}

/// Constructs a transaction that reveals the peg out payment address
pub fn withdrawal_request_reveal_unsigned(
    withdrawal_data: WithdrawalData,
    reveal_inputs: RevealInputs,
    fulfillment_fee: u64,
    commit_amount: u64,
    peg_wallet_address: BitcoinAddress,
    recipient_wallet_address: BitcoinAddress,
) -> CommitRevealResult<Transaction> {
    let mut tx = reveal(&withdrawal_data.to_vec()?, reveal_inputs)?;

    tx.output.push(TxOut {
        value: commit_amount - withdrawal_data.reveal_fee - fulfillment_fee,
        script_pubkey: recipient_wallet_address.script_pubkey(),
    });
    tx.output.push(TxOut {
        value: fulfillment_fee,
        script_pubkey: peg_wallet_address.script_pubkey(),
    });

    Ok(tx)
}

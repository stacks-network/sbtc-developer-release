/// sBTC blockchain manager module
pub mod blockchain;
/// sBTC signer configuration module
pub mod config;
/// sBTC coordinator module
pub mod coordinator;

use std::collections::HashMap;

use crate::{
    signer::blockchain::Broker, signer::config::Config, signer::coordinator::Coordinate, SBTCError,
    SBTCResult,
};
use bitcoin::{Address, Network, PublicKey, Transaction as BitcoinTransaction};
use p256k1::ecdsa;
use wsts::Scalar;

#[derive(Default, Clone, Debug)]
/// Signers' public keys required for weighted distributed signing
/// TODO: replace with Frost library's PublicKeys
pub struct PublicKeys {
    /// Signer ids to public key mapping
    pub signer_ids: HashMap<u32, ecdsa::PublicKey>,
    /// Vote ids to public key mapping
    pub vote_ids: HashMap<u32, ecdsa::PublicKey>,
}

/// A Stacks transaction
/// TODO: replace with the core library's StacksTransaction
pub struct StacksTransaction {}

/// A Frost signer
/// TODO: replace with the core library's FrostSigner
#[derive(Default)]
pub struct FrostSigner {}

impl Sign for FrostSigner {
    /// Sign the given message
    fn sign_message(&self, _message: &[u8]) -> SBTCResult<Vec<u8>> {
        todo!()
    }

    /// Verify the message was signed by the given public key
    fn verify_message(&self, _public_key: &ecdsa::PublicKey, _message: &[u8]) -> SBTCResult<bool> {
        todo!()
    }
}

/// An Bitcoin transaction needing to be SIGNED by the signer
/// TODO: update with https://github.com/Trust-Machines/stacks-sbtc/pull/595
pub enum SignableTransaction {
    /// A withdrawal fulfillment Bitcoin transaction
    WithdrawalFulfillment(BitcoinTransaction),
    /// A Bitcoin sBTC wallet handoff transaction
    Handoff(BitcoinTransaction),
}

/// sBTC Keys trait for retrieving signer IDs, vote IDs, and public keys
trait Keys {
    /// Retrieve the current public keys for the signers and their vote ids
    fn public_keys(&self) -> SBTCResult<PublicKeys>;
    /// Get the ordered list of coordinator public keys for the given transaction
    fn coordinator_public_keys(&self, tx: &BitcoinTransaction) -> SBTCResult<Vec<PublicKey>>;
}

/// Sign trait for signing and verifying messages
pub trait Sign {
    /// Sign the given message
    fn sign_message(&self, message: &[u8]) -> SBTCResult<Vec<u8>>;
    /// Verify the message was signed by the given public key
    fn verify_message(&self, public_key: &ecdsa::PublicKey, message: &[u8]) -> SBTCResult<bool>;
}

/// Validator trait for validating pending Bitcoin transactions
pub trait Validator {
    /// Validate the given signable Bitcoin transaction
    fn validate_transaction(&self, tx: &SignableTransaction) -> SBTCResult<bool>;
}

/// sBTC compliant Signer
pub struct Signer<S, C> {
    /// Signer configuration
    pub config: Config,
    /// Signer private key
    pub private_key: Scalar,
    /// Network to use
    pub network: Network,
    /// The blockchain Broker
    pub broker: Broker,
    /// The signer
    pub signer: S,
    /// The coordinator
    pub coordinator: C,
}

impl<S: Sign, C: Coordinate> Signer<S, C> {
    // Public methods

    /// Create a new signer
    pub fn new(
        config: Config,
        private_key: Scalar,
        network: Network,
        broker: Broker,
        signer: S,
        coordinator: C,
    ) -> Self {
        Self {
            config,
            private_key,
            network,
            broker,
            signer,
            coordinator,
        }
    }

    /// Run the signer. Will block until the signer is stopped.
    pub fn run(&self) -> SBTCResult<()> {
        todo!()
    }

    /// Sign approve the given transaction
    pub fn approve(&self, _tx: &BitcoinTransaction) -> SBTCResult<()> {
        todo!()
    }

    /// Sign deny the given transaction
    pub fn deny(&self, _tx: &BitcoinTransaction) -> Result<(), SBTCError> {
        todo!()
    }

    // Private methods

    /// Fulfill the withdrawal request using the provided address
    fn _fulfill_withdrawal_request(
        &self,
        _sbtc_wallet_address: &Address,
        _tx: &StacksTransaction,
    ) -> SBTCResult<()> {
        todo!()
    }
}

impl<S, C> Keys for Signer<S, C> {
    /// Retrieve the current public keys for the signers and their vote ids
    fn public_keys(&self) -> SBTCResult<PublicKeys> {
        self.broker.public_keys()
    }

    /// Get the ordered list of coordinator public keys for the given transaction
    fn coordinator_public_keys(&self, _tx: &BitcoinTransaction) -> SBTCResult<Vec<PublicKey>> {
        todo!()
    }
}

impl<S, C> Validator for Signer<S, C> {
    /// Validate the given sBTC transaction
    fn validate_transaction(&self, tx: &SignableTransaction) -> SBTCResult<bool> {
        // TODO: check all addresses involved in each transaction
        match tx {
            SignableTransaction::WithdrawalFulfillment(_tx) => {
                todo!()
            }
            SignableTransaction::Handoff(_tx) => {
                todo!()
            }
        }
    }
}

// Implement the `Sign` trait for `Signer` where the generic type `S` also implements `Sign`
impl<S: Sign, C> Sign for Signer<S, C> {
    /// Sign the given message
    fn sign_message(&self, message: &[u8]) -> SBTCResult<Vec<u8>> {
        self.signer.sign_message(message)
    }
    /// Verify the message was signed by the given public key
    fn verify_message(&self, public_key: &ecdsa::PublicKey, message: &[u8]) -> SBTCResult<bool> {
        self.signer.verify_message(public_key, message)
    }
}

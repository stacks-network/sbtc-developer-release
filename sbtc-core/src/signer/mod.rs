/// An sBTC signer configuration
pub mod config;

use std::collections::HashMap;

use crate::SBTCError;
use crate::{signer::config::Config, SBTCResult};
use bitcoin::{Address, Network, PrivateKey, PublicKey, Transaction as BitcoinTransaction};
use p256k1::ecdsa;
use url::Url;
use wsts::{bip340::SchnorrProof, common::Signature};

/// A Stacks transaction
/// TODO: replace with the core library's StacksTransaction
pub struct StacksTransaction {}

/// An sBTC transaction
/// TODO: replace with the core library's SBTCTransaction
/// This could be a BTC transaction or a STX transaction
/// depending on https://github.com/Trust-Machines/stacks-sbtc/pull/595
pub struct SBTCTransaction {}

#[derive(Default, Clone, Debug)]
/// Signers' public keys required for weighted distributed signing
pub struct PublicKeys {
    /// Signer ids to public key mapping
    pub signer_ids: HashMap<u32, ecdsa::PublicKey>,
    /// Vote ids to public key mapping
    pub vote_ids: HashMap<u32, ecdsa::PublicKey>,
}

/// sBTC Keys trait for retrieving signer IDs, vote IDs, and public keys
trait Keys {
    fn public_keys(&self) -> SBTCResult<PublicKeys>;
    fn coordinator_public_key(&self, tx: &BitcoinTransaction) -> SBTCResult<PublicKey>;
}

/// Coordinator trait for generating the sBTC wallet public key and running a signing round
pub trait Coordinate {
    fn generate_sbtc_wallet_public_key(&self, public_keys: &PublicKeys) -> SBTCResult<PublicKey>;
    fn run_signing_round(
        &self,
        public_keys: &PublicKeys,
        tx: &BitcoinTransaction,
    ) -> SBTCResult<(Signature, SchnorrProof)>;
}

/// Sign trait for signing and verifying messages
pub trait Sign {
    fn sign_message(&self, message: &[u8]) -> SBTCResult<Vec<u8>>;
    fn verify_message(&self, public_key: &ecdsa::PublicKey, message: &[u8]) -> SBTCResult<bool>;
}

/// sBTC compliant Signer
pub struct Signer<S: Sign + Coordinate> {
    /// Signer configuration
    pub config: Config,
    /// Signer private key
    pub private_key: PrivateKey,
    /// Network to use
    pub network: Network,
    /// The stacks node RPC URL
    pub stacks_node_rpc_url: Url,
    /// The bitcoin node RPC URL
    pub bitcoin_node_rpc_url: Url,
    /// The signer
    pub signer: S,
}

impl<S: Sign + Coordinate> Signer<S> {
    // Public methods

    /// Create a new signer
    pub fn new(
        config: Config,
        private_key: PrivateKey,
        network: Network,
        stacks_node_rpc_url: Url,
        bitcoin_node_rpc_url: Url,
        signer: S,
    ) -> Self {
        Self {
            config,
            private_key,
            network,
            stacks_node_rpc_url,
            bitcoin_node_rpc_url,
            signer,
        }
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

    /// Retrieve pending sBTC transactions
    fn _sbtc_transactions(&self) -> SBTCResult<Vec<SBTCTransaction>> {
        todo!()
    }

    /// Fulfill the withdrawal request using the provided address
    fn _fulfill_withdrawal_request(
        &self,
        _sbtc_wallet_address: &Address,
        _tx: &SBTCTransaction,
    ) -> SBTCResult<()> {
        todo!()
    }

    /// Broadcast the transaction to the bitcoin network
    fn _broadcast_transaction_bitcoin(&self, _tx: BitcoinTransaction) -> SBTCResult<()> {
        todo!()
    }

    /// Broadcast the transaction to the stacks network
    fn _broadcast_transaction_stacks(&self, _tx: StacksTransaction) -> SBTCResult<()> {
        todo!()
    }
}

impl<S: Sign + Coordinate> Keys for Signer<S> {
    /// Retrieve the current public keys for the signers and their vote ids
    fn public_keys(&self) -> SBTCResult<PublicKeys> {
        todo!()
    }

    /// Get the current coordinator public key for the given transaction
    /// TODO: this may have to be per reward cycle since the coordinator public key changes between reward cycles
    /// and the DKG round must be triggered by a coordinator without a specific BitcoinTransaction
    fn coordinator_public_key(&self, _tx: &BitcoinTransaction) -> SBTCResult<PublicKey> {
        todo!()
    }
}

impl<S: Sign + Coordinate> Coordinate for Signer<S> {
    /// Generate the sBTC wallet public key
    fn generate_sbtc_wallet_public_key(&self, _public_keys: &PublicKeys) -> SBTCResult<PublicKey> {
        todo!()
    }

    /// Run the signing round for the transaction
    fn run_signing_round(
        &self,
        _public_keys: &PublicKeys,
        _tx: &BitcoinTransaction,
    ) -> SBTCResult<(Signature, SchnorrProof)> {
        todo!()
    }
}

impl<S: Sign + Coordinate> Sign for Signer<S> {
    /// Sign the given message
    fn sign_message(&self, _message: &[u8]) -> SBTCResult<Vec<u8>> {
        todo!()
    }
    /// Verify the message was signed by the given public key
    /// TODO: replace ecdsa::PublicKey with a more generic type to enable a more generic signer
    fn verify_message(&self, _public_key: &ecdsa::PublicKey, _message: &[u8]) -> SBTCResult<bool> {
        todo!()
    }
}

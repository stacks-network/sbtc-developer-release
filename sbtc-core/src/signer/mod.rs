/// An sBTC signer configuration
pub mod config;

use std::collections::HashMap;

use crate::signer::config::Config;
use crate::SBTCError;
use bitcoin::{Network, PrivateKey, PublicKey, Transaction as BitcoinTransaction};
use p256k1::ecdsa;
use url::Url;
use wsts::{bip340::SchnorrProof, common::Signature};

#[derive(Default, Clone, Debug)]
pub struct SignersData {
    pub signers: HashMap<u32, ecdsa::PublicKey>,
    pub vote_shares: HashMap<u32, ecdsa::PublicKey>,
}

/// Coordinator algorithm trait
pub trait CoordinatorAlgorithm {
    fn generate_sbtc_wallet_public_key(&self) -> Result<PublicKey, SBTCError>;
    fn run_signing_round(
        &self,
        _tx: BitcoinTransaction,
    ) -> Result<(Signature, SchnorrProof), SBTCError>;
}

/// Signer algorithm trait
pub trait SignerAlgorithm: CoordinatorAlgorithm {
    fn sign_message(&self, message: &[u8]) -> Result<Vec<u8>, SBTCError>;
    fn verify_message(
        &self,
        public_key: &ecdsa::PublicKey,
        message: &[u8],
    ) -> Result<bool, SBTCError>;
}

/// sBTC compliant Signer
pub struct Signer<S: SignerAlgorithm> {
    /// Signer configuration
    pub config: Config,
    /// Signer private key
    pub private_key: PrivateKey,
    /// Network to use
    pub network: Network,
    /// The stacks node RPC URL
    pub stacks_node_rpc_url: Url,
    /// The signing algorithm
    pub algorithm: S,
}

impl<S: SignerAlgorithm> Signer<S> {
    /// Create a new signer
    pub fn new(
        config: Config,
        private_key: PrivateKey,
        network: Network,
        stacks_node_rpc_url: Url,
        algorithm: S,
    ) -> Self {
        Self {
            config,
            private_key,
            network,
            stacks_node_rpc_url,
            algorithm,
        }
    }

    /// Get the current coordinator public key from the stacks node
    pub fn get_coordinator_public_key(&self) -> Result<PublicKey, SBTCError> {
        todo!()
    }

    /// Retrieve the current signers' data from the stacks node
    pub fn get_signers_data(&self) -> Result<SignersData, SBTCError> {
        todo!()
    }

    /// Sign approve the given transaction
    pub fn approve(&self, _tx: BitcoinTransaction) -> Result<(), SBTCError> {
        todo!()
    }

    /// Sign deny the given transaction
    pub fn deny(&self, _tx: BitcoinTransaction) -> Result<(), SBTCError> {
        todo!()
    }
}

impl<S: SignerAlgorithm> CoordinatorAlgorithm for Signer<S> {
    /// Generate the sBTC wallet public key
    fn generate_sbtc_wallet_public_key(&self) -> Result<PublicKey, SBTCError> {
        todo!()
    }

    /// Run the signing round for the transaction
    fn run_signing_round(
        &self,
        _tx: BitcoinTransaction,
    ) -> Result<(Signature, SchnorrProof), SBTCError> {
        todo!()
    }
}

impl<S: SignerAlgorithm> SignerAlgorithm for Signer<S> {
    /// Sign the given message
    fn sign_message(&self, _message: &[u8]) -> Result<Vec<u8>, SBTCError> {
        todo!()
    }
    /// Verify the message was signed by the given public key
    fn verify_message(
        &self,
        _public_key: &ecdsa::PublicKey,
        _message: &[u8],
    ) -> Result<bool, SBTCError> {
        todo!()
    }
}

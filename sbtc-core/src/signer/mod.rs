/// An sBTC signer configuration
pub mod config;

use std::collections::HashMap;

use crate::SBTCError;
use crate::{signer::config::Config, SBTCResult};
use bitcoin::{Network, PrivateKey, PublicKey, Transaction as BitcoinTransaction};
use p256k1::ecdsa;
use url::Url;
use wsts::{bip340::SchnorrProof, common::Signature};

#[derive(Default, Clone, Debug)]
/// Signers' data required for weighted distributed signing
pub struct Signers {
    /// Signer id to public key mapping
    pub signer_ids: HashMap<u32, ecdsa::PublicKey>,
    /// Vote share to public key mapping
    pub vote_shares: HashMap<u32, ecdsa::PublicKey>,
}

/// sBTC Signer trait
pub trait SBTCSigner {
    fn signers(&self) -> SBTCResult<Signers>;
    fn coordinator_public_key(&self, _tx: BitcoinTransaction) -> SBTCResult<PublicKey>;
}

/// Coordinator trait
pub trait Coordinate {
    fn generate_sbtc_wallet_public_key(&self) -> SBTCResult<PublicKey>;
    fn run_signing_round(&self, _tx: BitcoinTransaction) -> SBTCResult<(Signature, SchnorrProof)>;
}

/// Sign trait
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
    /// The signer
    pub signer: S,
}

impl<S: Sign + Coordinate> Signer<S> {
    /// Create a new signer
    pub fn new(
        config: Config,
        private_key: PrivateKey,
        network: Network,
        stacks_node_rpc_url: Url,
        signer: S,
    ) -> Self {
        Self {
            config,
            private_key,
            network,
            stacks_node_rpc_url,
            signer,
        }
    }

    /// Sign approve the given transaction
    pub fn approve(&self, _tx: BitcoinTransaction) -> SBTCResult<()> {
        todo!()
    }

    /// Sign deny the given transaction
    pub fn deny(&self, _tx: BitcoinTransaction) -> Result<(), SBTCError> {
        todo!()
    }
}

impl<S: Sign + Coordinate> SBTCSigner for Signer<S> {
    /// Retrieve the current signers
    fn signers(&self) -> SBTCResult<Signers> {
        todo!()
    }

    /// Get the current coordinator public key for the given transaction
    fn coordinator_public_key(&self, _tx: BitcoinTransaction) -> SBTCResult<PublicKey> {
        todo!()
    }
}

impl<S: Sign + Coordinate> Coordinate for Signer<S> {
    /// Generate the sBTC wallet public key
    fn generate_sbtc_wallet_public_key(&self) -> SBTCResult<PublicKey> {
        todo!()
    }

    /// Run the signing round for the transaction
    fn run_signing_round(&self, _tx: BitcoinTransaction) -> SBTCResult<(Signature, SchnorrProof)> {
        todo!()
    }
}

impl<S: Sign + Coordinate> Sign for Signer<S> {
    /// Sign the given message
    fn sign_message(&self, _message: &[u8]) -> SBTCResult<Vec<u8>> {
        todo!()
    }
    /// Verify the message was signed by the given public key
    fn verify_message(&self, _public_key: &ecdsa::PublicKey, _message: &[u8]) -> SBTCResult<bool> {
        todo!()
    }
}

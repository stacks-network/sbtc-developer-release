/// Module for Frost Interactive Robustness Extension signature generation
pub mod fire;
/// Module for RObust Asynchronous Schnorr Threshold signature generation
pub mod roast;

use std::collections::HashMap;

use crate::SBTCResult;
use bitcoin::{PublicKey, Transaction as BitcoinTransaction};
use p256k1::ecdsa;
use wsts::{bip340::SchnorrProof, common::Signature};

#[derive(Default, Clone, Debug)]
/// Signers' public keys required for weighted distributed signing
pub struct PublicKeys {
    /// Signer ids to public key mapping
    pub signer_ids: HashMap<u32, ecdsa::PublicKey>,
    /// Vote ids to public key mapping
    pub vote_ids: HashMap<u32, ecdsa::PublicKey>,
}

/// TODO: Define the Message types for DKG round
/// https://github.com/stacks-network/sbtc/issues/42

/// TODO: Define the Message types for Tx Signning Round
/// https://github.com/stacks-network/sbtc/issues/43

/// Coordinator trait for generating the sBTC wallet public key and running a signing round
pub trait Coordinate {
    /// Generate the sBTC wallet public key
    fn generate_sbtc_wallet_public_key(&self, public_keys: &PublicKeys) -> SBTCResult<PublicKey>;
    /// Run the signing round for the transaction
    fn run_signing_round(
        &self,
        public_keys: &PublicKeys,
        tx: &BitcoinTransaction,
    ) -> SBTCResult<(Signature, SchnorrProof)>;
}

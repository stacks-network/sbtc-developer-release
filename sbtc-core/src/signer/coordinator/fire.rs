// TODO: FIRE coordination logic
// https://github.com/Trust-Machines/stacks-sbtc/issues/667
use async_trait::async_trait;
use crate::signer::{
    coordinator::{Coordinate, PublicKeys},
    SBTCResult,
};
use bitcoin::{PublicKey, Transaction as BitcoinTransaction};
use wsts::{bip340::SchnorrProof, common::Signature};
/// FIRE coordinator

#[derive(Default)]
pub struct Coordinator {}

#[async_trait]
impl Coordinate for Coordinator {
    /// Generate the sBTC wallet public key
    async fn generate_sbtc_wallet_public_key(
        &self,
        _public_keys: &PublicKeys,
    ) -> SBTCResult<PublicKey> {
        todo!()
    }
    /// Run the signing round for the transaction
    async fn run_signing_round(
        &self,
        _public_keys: &PublicKeys,
        _tx: &BitcoinTransaction,
    ) -> SBTCResult<(Signature, SchnorrProof)> {
        todo!()
    }
}

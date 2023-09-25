/// Module for Frost Interactive Robustness Extension signature generation
pub mod fire;
/// Module for RObust Asynchronous Schnorr Threshold signature generation
pub mod roast;

use std::collections::HashMap;

use bdk::bitcoin::{
	util::taproot::TaprootSpendInfo, PublicKey,
	Transaction as BitcoinTransaction,
};
use p256k1::ecdsa;
use wsts::{bip340::SchnorrProof, common::Signature};

use super::StacksTransaction;
use crate::SBTCResult;

#[derive(Default, Clone, Debug)]
/// Signers' public keys required for weighted distributed signing
pub struct PublicKeys {
	/// Signer ids to public key mapping
	pub signer_ids: HashMap<u32, ecdsa::PublicKey>,
	/// Vote ids to public key mapping
	pub vote_ids: HashMap<u32, ecdsa::PublicKey>,
}

/// TODO: Define the Message types for DKG round
/// <https://github.com/stacks-network/sbtc/issues/42>

/// TODO: Define the Message types for Tx Signning Round
/// <https://github.com/stacks-network/sbtc/issues/43>

/// An sBTC transaction needing to be processed by the coordinator
/// TODO: replace with the core library's SBTCTransaction
/// This could be a BTC transaction or a STX transaction
/// depending on https://github.com/Trust-Machines/stacks-sbtc/pull/595
pub enum SBTCTransaction {
	/// A commit Bitcoin transaction
	Commit(TaprootSpendInfo, BitcoinTransaction),
	/// A withdrawal Stacks transaction
	Withdawal(StacksTransaction),
}

/// Revealer trait for revealing BTC commit transactions
pub trait Reveal {
	/// Retrieve Commit transactions from Revealer service
	fn commit_transactions(
		&self,
	) -> SBTCResult<Vec<(TaprootSpendInfo, BitcoinTransaction)>>;
	/// Validate the given commit transaction
	fn validate_commit_transaction(
		&self,
		spend_info: TaprootSpendInfo,
		tx: &BitcoinTransaction,
	) -> SBTCResult<bool>;
	/// Create a reveal transaction from the BTC commit transaction
	fn reveal_transaction(
		&self,
		spend_info: TaprootSpendInfo,
		tx: &BitcoinTransaction,
	) -> SBTCResult<BitcoinTransaction>;
}

/// Coordinator trait for generating the sBTC wallet public key and running a
/// signing round
pub trait Coordinate {
	/// Retrieve sBTC transactions from the blockchain
	fn sbtc_transactions(&self) -> SBTCResult<Vec<SBTCTransaction>>;
	/// Generate the sBTC wallet public key
	fn generate_sbtc_wallet_public_key(
		&self,
		public_keys: &PublicKeys,
	) -> SBTCResult<PublicKey>;
	/// Run the signing round for the transaction
	fn run_signing_round(
		&self,
		public_keys: &PublicKeys,
		tx: &BitcoinTransaction,
	) -> SBTCResult<(Signature, SchnorrProof)>;
}

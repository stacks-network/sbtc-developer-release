/// sBTC signer configuration module
pub mod config;
/// sBTC coordinator module
pub mod coordinator;

use crate::SBTCError;
use crate::{
    signer::config::Config,
    signer::coordinator::{Coordinate, PublicKeys},
    SBTCResult,
};
use bitcoin::{Address, Network, PrivateKey, PublicKey, Transaction as BitcoinTransaction};
use p256k1::ecdsa;
use url::Url;

/// A Stacks transaction
/// TODO: replace with the core library's StacksTransaction
pub struct StacksTransaction {}

/// An sBTC transaction
/// TODO: replace with the core library's SBTCTransaction
/// This could be a BTC transaction or a STX transaction
/// depending on https://github.com/Trust-Machines/stacks-sbtc/pull/595
pub struct SBTCTransaction {}

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

    /// Get the ordered list of coordinator public keys for the given transaction
    fn coordinator_public_keys(&self, _tx: &BitcoinTransaction) -> SBTCResult<Vec<PublicKey>> {
        todo!()
    }
}

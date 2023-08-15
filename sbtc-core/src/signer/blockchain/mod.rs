use bitcoin::Transaction as BitcoinTransaction;
use url::Url;

use crate::{
    signer::{PublicKeys, StacksTransaction},
    SBTCResult,
};

/// The broker is responsible for retreiving sBTC withdrawals requests and
/// broadcasting transactions to the stacks and bitcoin networks.
pub struct Broker {
    /// The bitcoin node RPC URL
    pub bitcoin_node_rpc_url: Url,
    /// The stacks node RPC URL
    pub stacks_node_rpc_url: Url,
}

impl Broker {
    /// Create a new broker
    pub fn new(bitcoin_node_rpc_url: Url, stacks_node_rpc_url: Url) -> Self {
        Self {
            bitcoin_node_rpc_url,
            stacks_node_rpc_url,
        }
    }

    /// Retrieve the current public keys for the signers and their vote ids from the stacks node
    pub fn public_keys(&self) -> SBTCResult<PublicKeys> {
        todo!()
    }

    /// Retrieve withdrawal transactions from the stacks node
    pub fn withdrawal_transactions(&self) -> SBTCResult<Vec<StacksTransaction>> {
        todo!()
    }

    /// Broadcast the transaction to the bitcoin network
    pub fn broadcast_transaction_bitcoin(&self, _tx: BitcoinTransaction) -> SBTCResult<()> {
        todo!()
    }

    /// Broadcast the transaction to the stacks network
    pub fn broadcast_transaction_stacks(&self, _tx: StacksTransaction) -> SBTCResult<()> {
        todo!()
    }
}

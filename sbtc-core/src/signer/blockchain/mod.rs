use bitcoin::{util::taproot::TaprootSpendInfo, Transaction as BitcoinTransaction};
use url::Url;

use crate::{
    signer::{PublicKeys, StacksTransaction},
    SBTCResult,
};

/// An sBTC transaction needing to be processed
/// TODO: replace with the core library's SBTCTransaction
/// This could be a BTC transaction or a STX transaction
/// depending on https://github.com/Trust-Machines/stacks-sbtc/pull/595
pub enum SBTCTransaction {
    /// A commit Bitcoin transaction
    Commit(TaprootSpendInfo, BitcoinTransaction),
    /// A withdrawal Stacks transaction
    Withdrawal(StacksTransaction),
}

/// The broker is responsible for retreiving sBTC withdrawals and deposit
/// requests and broadcasting transactions to the stacks and bitcoin networks.
pub struct Broker {
    /// The revealer RPC URL
    pub revealer_rpc_url: Url,
    /// The bitcoin node RPC URL
    pub bitcoin_node_rpc_url: Url,
    /// The stacks node RPC URL
    pub stacks_node_rpc_url: Url,
}

impl Broker {
    /// Create a new broker
    pub fn new(revealer_rpc_url: Url, bitcoin_node_rpc_url: Url, stacks_node_rpc_url: Url) -> Self {
        Self {
            revealer_rpc_url,
            bitcoin_node_rpc_url,
            stacks_node_rpc_url,
        }
    }

    /// Retrieve the current public keys for the signers and their vote ids from the stacks node
    pub fn public_keys(&self) -> SBTCResult<PublicKeys> {
        todo!()
    }

    /// Retrieve sBTC transactions from the stacks and bitcoin nodes and revealer service
    pub fn sbtc_transactions(&self) -> SBTCResult<Vec<SBTCTransaction>> {
        let mut sbtc_transactions = Vec::new();
        let commit_transactions = self.commit_transactions()?;
        for tx in self.withdrawal_transactions()? {
            sbtc_transactions.push(SBTCTransaction::Withdrawal(tx));
        }
        for (spend_info, commit_tx) in commit_transactions {
            sbtc_transactions.push(SBTCTransaction::Commit(spend_info, commit_tx));
        }
        Ok(sbtc_transactions)
    }

    /// Broadcast the transaction to the bitcoin network
    pub fn broadcast_transaction_bitcoin(&self, _tx: BitcoinTransaction) -> SBTCResult<()> {
        todo!()
    }

    /// Broadcast the transaction to the stacks network
    pub fn broadcast_transaction_stacks(&self, _tx: StacksTransaction) -> SBTCResult<()> {
        todo!()
    }

    /// Helper function to retrieve commit transactions from Revealer service
    fn commit_transactions(&self) -> SBTCResult<Vec<(TaprootSpendInfo, BitcoinTransaction)>> {
        todo!()
    }

    /// Helper function to retrieve withdrawal transactions from the stacks node
    fn withdrawal_transactions(&self) -> SBTCResult<Vec<StacksTransaction>> {
        todo!()
    }
}

use bitcoin::{Address as BitcoinAddress, Transaction as BitcoinTransaction};
use p256k1::ecdsa;
use stacks_core::utils::PrincipalData;
use std::collections::HashMap;
use url::Url;

use crate::{
    signer::{self, StacksTransaction},
    SBTCResult,
};

/// Placeholder for important data for a speific signer in a specific cycle
struct SignerData {
    /// The amount stacked in the cycle
    amount: u64,
    /// the locked stacks balance
    vote: Option<bool>,
    /// the height at which the locked stacks unlocks
    public_key: ecdsa::PublicKey,
}

/// Placeholder for important data from the specific cycle
struct StackerData {
    /// All the stackers for the specific cycle
    stackers: Vec<PrincipalData>,
    /// All the STX stacked for the specific cycle
    stacked: u64,
    /// The current sBTC wallet address
    _sbtc_wallet_address: BitcoinAddress,
}

trait ReadOnlyCallable {
    /// Helper function for calling read-only functions on the smart contract
    fn read_only_function(
        &self,
        block_height: u64,
        function_name: &str,
        function_args: &[&str],
    ) -> SBTCResult<String>;

    /// Helper function for calling get-specific-cycle-pool
    fn specific_cycle_pool(&self, block_height: u64, cycle: u64) -> SBTCResult<StackerData>;

    /// Helper function for calling get-current-cycle-pool
    fn current_cycle_pool(&self, block_height: u64) -> SBTCResult<u64>;

    /// Helper function for calling get-signer-in-cycle
    fn signer_in_cycle(
        &self,
        block_height: u64,
        stx_principal: &PrincipalData,
        cycle: u64,
    ) -> SBTCResult<SignerData>;

    /// Helper function for calling get-current-pre-signer
    fn current_signer(
        &self,
        block_height: u64,
        stx_principal: &PrincipalData,
        cycle: u64,
    ) -> SBTCResult<bool>;
    /// Helper function for calling get-current-pre-signer
    fn current_pre_signer(
        &self,
        block_height: u64,
        stx_principal: &PrincipalData,
        cycle: u64,
    ) -> SBTCResult<bool>;
}

trait Callable {
    /// Build a stacks transaction to call a smart contract function
    fn build_stacks_transaction(
        &self,
        function_name: impl Into<String>,
        function_args: &[&impl Into<String>],
        nonce: u64,
    ) -> SBTCResult<StacksTransaction>;

    /// Helper function for calling signer-register
    fn signer_register(
        &self,
        block_height: u64,
        stx_principal: &PrincipalData,
        amount: u64,
        btc_reward_address: BitcoinAddress,
    ) -> SBTCResult<()>;

    /// Helper function for calling signer-pre-register
    fn signer_pre_register(
        &self,
        block_height: u64,
        stx_principal: &PrincipalData,
        amount: u64,
        btc_reward_address: BitcoinAddress,
    ) -> SBTCResult<()>;

    /// Helper function for calling vote-for-threshold-wallet-candidate
    fn vote_for_threshold_wallet_candidate(
        &self,
        block_height: u64,
        stx_principal: &PrincipalData,
        btc_reward_address: BitcoinAddress,
    ) -> SBTCResult<()>;
}

/// The broker is responsible reading info from and broadcasting transactions to the stacks and bitcoin networks.
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

    /// Retrieve the current public keys for the signers and their vote ids from the smart contract
    pub fn signer_public_keys(
        &self,
        block_height: u64,
        cycle: u64,
    ) -> SBTCResult<signer::PublicKeys> {
        let cycle_data = self.specific_cycle_pool(block_height, cycle)?;
        let mut vote_ids = HashMap::new();
        let mut signer_ids = HashMap::new();
        for (signer_id, stacker) in cycle_data.stackers.iter().enumerate() {
            let signer_data = self.signer_in_cycle(block_height, stacker, cycle)?;
            let vote_share =
                (signer_data.amount as f64 / cycle_data.stacked as f64 * 4000.0) as u32;
            let public_key = signer_data.public_key;
            for vote_id in 0..vote_share {
                vote_ids.insert(vote_id, public_key);
            }
            signer_ids.insert(signer_id as u32, public_key);
        }
        Ok(signer::PublicKeys {
            vote_ids,
            signer_ids,
        })
    }

    /// Retrieve withdrawal transactions from the smart contract
    pub fn pending_withdrawal_transactions(&self) -> SBTCResult<Vec<StacksTransaction>> {
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

    /// Register the signer
    pub fn register_signer(&self) -> SBTCResult<()> {
        todo!()
    }

    /// Pre-register the signer
    pub fn pre_register_signer(&self) -> SBTCResult<()> {
        todo!()
    }

    /// Register the provided BTC address as a vote for the threshold wallet
    pub fn register_sbtc_wallet_address_vote(
        &self,
        _btc_address: BitcoinAddress,
    ) -> SBTCResult<()> {
        todo!()
    }
}

impl ReadOnlyCallable for Broker {
    /// Call a read only function in the smart contract
    fn read_only_function(
        &self,
        _block_height: u64,
        _function_name: &str,
        _function_args: &[&str],
    ) -> SBTCResult<String> {
        todo!("construct a read only function call and return the unparsed response")
    }

    /// Helper function for calling get-specific-cycle-pool
    fn specific_cycle_pool(&self, _block_height: u64, _cycle: u64) -> SBTCResult<StackerData> {
        todo!("call read only function for get-specific-cycle-pool and parse the response")
    }

    /// Helper function for calling get-current-cycle-pool
    fn current_cycle_pool(&self, _block_height: u64) -> SBTCResult<u64> {
        todo!("call read only function for get-current-cycle-pool and parse the response")
    }

    /// Helper function for calling get-signer-in-cycle
    fn signer_in_cycle(
        &self,
        _block_height: u64,
        _stx_principal: &PrincipalData,
        _cycle: u64,
    ) -> SBTCResult<SignerData> {
        todo!("call read only function for get-signer-in-cycle and parse the response")
    }

    /// Helper function for calling get-current-pre-signer
    fn current_signer(
        &self,
        _block_height: u64,
        _stx_principal: &PrincipalData,
        _cycle: u64,
    ) -> SBTCResult<bool> {
        todo!("call read only function for get-current-signer and parse the response")
    }

    /// Helper function for calling get-current-pre-signer
    fn current_pre_signer(
        &self,
        _block_height: u64,
        _stx_principal: &PrincipalData,
        _cycle: u64,
    ) -> SBTCResult<bool> {
        todo!("call read only function for get-current-pre-signer and parse the response")
    }
}

impl Callable for Broker {
    /// Build a stacks transaction to call a smart contract function
    fn build_stacks_transaction(
        &self,
        _function_name: impl Into<String>,
        _function_args: &[&impl Into<String>],
        _nonce: u64,
    ) -> SBTCResult<StacksTransaction> {
        todo!()
    }

    /// Helper function for calling signer-register
    fn signer_register(
        &self,
        block_height: u64,
        stx_principal: &PrincipalData,
        _amount: u64,
        _btc_reward_address: BitcoinAddress,
    ) -> SBTCResult<()> {
        let cycle = self.current_cycle_pool(block_height)?;
        if self.current_signer(block_height, stx_principal, cycle)? {
            return Ok(());
        }
        todo!("call build transactions for signer-register and broadcast the result")
    }

    /// Helper function for calling signer-pre-register
    fn signer_pre_register(
        &self,
        block_height: u64,
        stx_principal: &PrincipalData,
        _amount: u64,
        _btc_reward_address: BitcoinAddress,
    ) -> SBTCResult<()> {
        let cycle = self.current_cycle_pool(block_height)?;
        if self.current_pre_signer(block_height, stx_principal, cycle)? {
            return Ok(());
        }
        todo!("call build transactions for signer-pre-register and broadcast the result")
    }

    /// Helper function for calling vote-for-threshold-wallet-candidate
    fn vote_for_threshold_wallet_candidate(
        &self,
        block_height: u64,
        stx_principal: &PrincipalData,
        _btc_reward_address: BitcoinAddress,
    ) -> SBTCResult<()> {
        let cycle = self.current_cycle_pool(block_height)?;
        if self
            .signer_in_cycle(block_height, stx_principal, cycle)?
            .vote
            .is_some()
        {
            return Ok(());
        }
        todo!("call build transactions for vote-for-threshold-wallet-candidate and broadcast the result")
    }
}

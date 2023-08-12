use bitcoin::{Address as BitcoinAddress, Network};
use serde::Deserialize;
use stacks_core::address::StacksAddress;
use std::str::FromStr;
use toml;

use crate::{SBTCError, SBTCResult};

#[derive(Clone, Debug)]
/// Configuration for the signer approval/denial
pub struct Config {
    /// The maximum dollar amount of a transaction that will be auto approved.
    pub auto_approve_max_amount: Option<u64>,
    /// The BTC addresses to be auto denied
    pub auto_deny_addresses_btc: Vec<BitcoinAddress>,
    /// The STX addresses to be auto denied
    pub auto_deny_addresses_stx: Vec<StacksAddress>,
    /// The number of blocks before deadline at which point the transaction will be auto denied. Default is 10 blocks.
    pub auto_deny_deadline_blocks: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_approve_max_amount: None,
            auto_deny_addresses_btc: vec![],
            auto_deny_addresses_stx: vec![],
            auto_deny_deadline_blocks: 10,
        }
    }
}

impl Config {
    /// Load the configuration from a toml file
    pub fn from_path(path: impl AsRef<std::path::Path>) -> SBTCResult<Self> {
        let config_toml = ConfigTOML::from_path(path)?;
        Self::try_from(config_toml)
    }
}

impl TryFrom<ConfigTOML> for Config {
    type Error = SBTCError;
    fn try_from(config: ConfigTOML) -> SBTCResult<Self> {
        let auto_approve_max_amount = config.auto_approve_max_amount;
        let auto_deny_addresses_btc = config
            .auto_deny_addresses_btc
            .unwrap_or_default()
            .into_iter()
            .filter_map(|address| BitcoinAddress::from_str(&address).ok())
            .collect();
        let auto_deny_addresses_stx = config
            .auto_deny_addresses_stx
            .unwrap_or_default()
            .into_iter()
            .filter_map(|address| StacksAddress::try_from(address.as_str()).ok())
            .collect();
        let auto_deny_deadline_blocks = config.auto_deny_deadline_blocks.unwrap_or(10);
        Ok(Self {
            auto_approve_max_amount,
            auto_deny_addresses_btc,
            auto_deny_addresses_stx,
            auto_deny_deadline_blocks,
        })
    }
}

#[derive(Clone, Deserialize, Debug)]
/// TOML Configuration for the signer
pub struct ConfigTOML {
    /// The private key of the signer
    pub private_key: String,
    /// The RPC Url of the stacks node
    pub stacks_node_rpc_url: String,
    /// The RPC Url of the bitcoin node
    pub bitcoin_node_rpc_url: String,
    /// The RPC URL of the revealer
    pub revealer_rpc_url: String,
    /// The network version we are using (One of 'Signet', 'Regtest', 'Testnet', 'Bitcoin'). Default: 'Testnet'
    pub network: Option<Network>,
    /// The maximum dollar amount of a transaction that will be auto approved.
    pub auto_approve_max_amount: Option<u64>,
    /// The BTC addresses to be auto denied
    pub auto_deny_addresses_btc: Option<Vec<String>>,
    /// The STX addresses to be auto denied
    pub auto_deny_addresses_stx: Option<Vec<String>>,
    /// The number of blocks before deadline at which point the transaction will be auto denied. Default is 10 blocks.
    pub auto_deny_deadline_blocks: Option<u32>,
}

impl ConfigTOML {
    /// Load the configuration from a toml file
    pub fn from_path(path: impl AsRef<std::path::Path>) -> SBTCResult<ConfigTOML> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| SBTCError::InvalidConfig(format!("Could not read config file: {}", e)))?;
        toml::from_str(&content)
            .map_err(|e| SBTCError::InvalidConfig(format!("Could not parse config file: {}", e)))
    }
}

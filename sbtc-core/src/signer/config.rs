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
        let mut auto_deny_addresses_btc = Vec::new();
        let mut auto_deny_addresses_stx = Vec::new();
        for address in config.auto_deny_addresses_btc.unwrap_or_default() {
            auto_deny_addresses_btc.push(BitcoinAddress::from_str(&address).map_err(|e| {
                SBTCError::InvalidConfig(format!("Failed to parse bitcoin address: {}", e))
            })?);
        }
        for address in config.auto_deny_addresses_stx.unwrap_or_default() {
            auto_deny_addresses_stx.push(StacksAddress::try_from(address.as_str()).map_err(
                |e| SBTCError::InvalidConfig(format!("Failed to parse stacks address: {}", e)),
            )?);
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempdir::TempDir;

    /// Helper function of writing a new config toml file and loading it into a ConfigTOML struct
    fn write_new_config_toml(
        private_key: String,
        stacks_node_rpc_url: String,
        bitcoin_node_rpc_url: String,
        revealer_rpc_url: String,
        network: Option<String>,
        auto_approve_max_amount: Option<u64>,
        auto_deny_addresses_btc: Option<Vec<String>>,
        auto_deny_addresses_stx: Option<Vec<String>>,
        auto_deny_deadline_blocks: Option<u32>,
    ) -> SBTCResult<ConfigTOML> {
        let dir = TempDir::new("").unwrap();
        let file_path = dir.path().join("signer.toml");
        let mut signer_file = std::fs::File::create(&file_path).unwrap();
        let mut signer_contents = format!(
            r#"
private_key = "{private_key}"
stacks_node_rpc_url = "{stacks_node_rpc_url}"
bitcoin_node_rpc_url = "{bitcoin_node_rpc_url}"
revealer_rpc_url = "{revealer_rpc_url}"
"#
        );
        if let Some(network) = network {
            signer_contents = format!("{signer_contents}\nnetwork = \"{network}\"");
        }
        if let Some(auto_approve_max_amount) = auto_approve_max_amount {
            signer_contents = format!(
                "{signer_contents}\nauto_approve_max_amount = \"{auto_approve_max_amount}\""
            );
        }
        if let Some(auto_deny_addresses_btc) = auto_deny_addresses_btc {
            let mut addresses = String::new();
            for (i, address) in auto_deny_addresses_btc.iter().enumerate() {
                addresses = if i == 0 {
                    format!("\"{address}\"")
                } else {
                    format!("{addresses}\n,\"{address}\"")
                };
            }
            signer_contents = format!("{signer_contents}\nauto_deny_addresses_btc = [{addresses}]");
        }
        if let Some(auto_deny_addresses_stx) = auto_deny_addresses_stx {
            let mut addresses = String::new();
            for (i, address) in auto_deny_addresses_stx.iter().enumerate() {
                addresses = if i == 0 {
                    format!("\"{address}\"")
                } else {
                    format!("{addresses}\n,\"{address}\"")
                };
            }
            signer_contents = format!("{signer_contents}\nauto_deny_addresses_stx = [{addresses}]");
        }
        if let Some(auto_deny_deadline_blocks) = auto_deny_deadline_blocks {
            signer_contents = format!(
                "{signer_contents}\nauto_deny_deadline_blocks = \"{auto_deny_deadline_blocks}\""
            );
        }
        println!("{}", signer_contents);
        signer_file.write_all(signer_contents.as_bytes()).unwrap();
        ConfigTOML::from_path(file_path)
    }

    #[test]
    fn config_toml_should_succeed_from_valid_toml() {
        let config_toml = write_new_config_toml(
            "private_key".to_string(),
            "stacks_node_rpc_url".to_string(),
            "bitcoin_node_rpc_url".to_string(),
            "revealer_rpc_url".to_string(),
            None,
            None,
            None,
            None,
            None,
        );
        assert!(config_toml.is_ok());
    }

    #[test]
    fn config_toml_should_fail_for_invalid_network() {
        let config_toml = write_new_config_toml(
            "private_key".to_string(),
            "stacks_node_rpc_url".to_string(),
            "bitcoin_node_rpc_url".to_string(),
            "revealer_rpc_url".to_string(),
            Some("invalid_network".to_string()),
            None,
            None,
            None,
            None,
        );
        assert!(config_toml.is_err());
    }

    #[test]
    fn config_should_succeed_for_valid_toml() {
        let config_toml = write_new_config_toml(
            "private_key".to_string(),
            "stacks_node_rpc_url".to_string(),
            "bitcoin_node_rpc_url".to_string(),
            "revealer_rpc_url".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("Failed to create config toml");
        let config = Config::try_from(config_toml);
        assert!(config.is_ok());
    }

    #[test]
    fn config_should_fail_for_invalid_bitcoin_addresses() {
        let config_toml = write_new_config_toml(
            "private_key".to_string(),
            "stacks_node_rpc_url".to_string(),
            "bitcoin_node_rpc_url".to_string(),
            "revealer_rpc_url".to_string(),
            None,
            None,
            Some(vec!["Invalid address".to_string()]),
            None,
            None,
        )
        .expect("Failed to create config toml");
        let config = Config::try_from(config_toml);
        assert!(config.is_err());
    }

    #[test]
    fn config_should_fail_for_invalid_stacks_addresses() {
        let config_toml = write_new_config_toml(
            "private_key".to_string(),
            "stacks_node_rpc_url".to_string(),
            "bitcoin_node_rpc_url".to_string(),
            "revealer_rpc_url".to_string(),
            None,
            None,
            None,
            Some(vec!["Invalid address".to_string()]),
            None,
        )
        .expect("Failed to create config toml");
        let config = Config::try_from(config_toml);
        assert!(config.is_err());
    }
}

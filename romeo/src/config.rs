//! Config

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use bdk::bitcoin::PrivateKey;
use blockstack_lib::types::chainstate::{StacksPrivateKey, StacksPublicKey};
use clap::Parser;

/// sBTC Alpha Romeo
#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Where the config file is located
    #[arg(short, long, value_name = "FILE")]
    pub config_file: PathBuf,
}

/// System configuration. This is typically constructed once and never mutated throughout the systems lifetime.
#[derive(Debug, Clone)]
pub struct Config {
    /// Directory to persist the state of the system to
    pub state_directory: PathBuf,

    /// Path to the contract file
    pub contract: PathBuf,

    /// Private key used for Bitcoin and Stacks transactions
    pub private_key: PrivateKey,

    /// Address of a bitcoin node
    pub bitcoin_node_url: reqwest::Url,

    /// Address of a stacks node
    pub stacks_node_url: reqwest::Url,

    /// Fee to use for stacks transactions
    pub stacks_transaction_fee: u64,

    /// Fee to use for bitcoin transactions
    pub bitcoin_transaction_fee: u64,
}

impl Config {
    /// Read the config file in the path
    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let config_root = normalize(
            std::env::current_dir().unwrap(),
            path.as_ref().parent().unwrap(),
        );

        let config_file = ConfigFile::from_path(&path)?;

        let state_directory = normalize(config_root.clone(), config_file.state_directory);

        let contract = normalize(config_root, config_file.contract);

        let private_key = PrivateKey::from_wif(&config_file.wif)?;

        let bitcoin_node_url = reqwest::Url::parse(&config_file.bitcoin_node_url)?;

        let stacks_node_url = reqwest::Url::parse(&config_file.stacks_node_url)?;

        Ok(Self {
            state_directory,
            contract,
            private_key,
            bitcoin_node_url,
            stacks_node_url,
            stacks_transaction_fee: config_file.stacks_transaction_fee,
            bitcoin_transaction_fee: config_file.bitcoin_transaction_fee,
        })
    }

    /// Stacks version of the private key
    pub fn stacks_private_key(&self) -> StacksPrivateKey {
        let mut pk = StacksPrivateKey::from_slice(&self.private_key.to_bytes()).unwrap();
        pk.set_compress_public(self.private_key.compressed);

        pk
    }

    /// Stacks public key corresponding to the private key
    pub fn stacks_public_key(&self) -> StacksPublicKey {
        StacksPublicKey::from_private(&self.stacks_private_key())
    }
}

fn normalize(root_dir: PathBuf, path: impl AsRef<Path>) -> PathBuf {
    if path.as_ref().is_relative() {
        root_dir.join(path)
    } else {
        path.as_ref().into()
    }
}

/// Network
#[derive(Debug, Clone, Copy)]
pub enum Network {
    /// Mainnet
    Mainnet,
    /// Testnet
    Testnet,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ConfigFile {
    /// Directory to persist the state of the system to
    pub state_directory: PathBuf,

    /// Path to the contract file
    pub contract: PathBuf,

    /// Private key used for Bitcoin and Stacks transactions
    pub wif: String,

    /// Address of a bitcoin node
    pub bitcoin_node_url: String,

    /// Address of a stacks node
    pub stacks_node_url: String,

    /// Fee to use for stacks transactions
    pub stacks_transaction_fee: u64,

    /// Fee to use for bitcoin transactions
    pub bitcoin_transaction_fee: u64,
}

impl ConfigFile {
    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let config_file = File::open(&path)?;

        Ok(serde_json::from_reader(config_file)?)
    }
}

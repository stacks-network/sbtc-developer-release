use std::{fs::File, path::PathBuf};

use clap::Parser;

/// sBTC Alpha Romeo
#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Where the config file is located
    #[arg(short, long, value_name = "FILE")]
    pub config_file: PathBuf,
}

/// System configuration. This is typically deserialized once and never mutated throughout the systems lifetime.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    /// Directory to persist the state of the system to
    pub state_directory: PathBuf,

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

impl Config {
    /// Read the config file specified in the CLI args
    pub fn from_args(args: Cli) -> anyhow::Result<Self> {
        let config_file = File::open(&args.config_file)?;
        let mut config: Self = serde_json::from_reader(config_file)?;

        if config.state_directory.is_relative() {
            config.state_directory = args
                .config_file
                .parent()
                .unwrap()
                .join(&config.state_directory);
        };

        Ok(config)
    }
}

use std::{fs::File, path::PathBuf};

use bdk::bitcoin::PrivateKey;
use clap::Parser;
use serde::Deserialize;

/// sBTC Alpha Romeo
#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Where the config file is located
    #[arg(short, long, value_name = "FILE")]
    pub config_file: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub state_directory: PathBuf,
    pub private_key: PrivateKey, // Used for both Bitcoin and Stacks transactions
}

impl Config {
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

use std::path::PathBuf;

use clap::Parser;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub state_directory: PathBuf,
    pub tick_interval_seconds: u64,
}

/// sBTC Alpha Romeo
#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Where the config file is located
    #[arg(short, long, value_name = "FILE")]
    pub config_file: PathBuf,
}

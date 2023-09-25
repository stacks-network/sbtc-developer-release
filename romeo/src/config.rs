//! Config

use std::{
	fs::File,
	path::{Path, PathBuf},
};

use bdk::bitcoin::Network as BitcoinNetwork;
use blockstack_lib::vm::ContractName;
use clap::Parser;
use stacks_core::{
	wallet::{BitcoinCredentials, Credentials, Wallet},
	Network as StacksNetwork,
};
use url::Url;

/// sBTC Alpha Romeo
#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
	/// Where the config file is located
	#[arg(short, long, value_name = "FILE")]
	pub config_file: PathBuf,
}

/// System configuration. This is typically constructed once and never mutated
/// throughout the systems lifetime.
#[derive(Debug, Clone)]
pub struct Config {
	/// Directory to persist the state of the system to
	pub state_directory: PathBuf,

	/// Credentials used to interact with the Stacks network
	pub stacks_credentials: Credentials,

	/// Credentials used to interact with the Bitcoin network
	pub bitcoin_credentials: BitcoinCredentials,

	/// Address of a stacks node
	pub stacks_node_url: Url,

	/// Address of a bitcoin node
	pub bitcoin_node_url: Url,

	/// sBTC asset contract name
	pub contract_name: ContractName,
}

impl Config {
	/// Read the config file in the path
	pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
		let config_root = normalize(
			std::env::current_dir().unwrap(),
			path.as_ref().parent().unwrap(),
		);

		let config_file = ConfigFile::from_path(&path)?;
		let state_directory =
			normalize(config_root.clone(), config_file.state_directory);

		let stacks_node_url = Url::parse(&config_file.stacks_node_url)?;
		let bitcoin_node_url = Url::parse(&config_file.bitcoin_node_url)?;

		let wallet = Wallet::new(&config_file.mnemonic)?;

		let stacks_credentials =
			wallet.credentials(config_file.stacks_network, 0)?;
		let bitcoin_credentials =
			wallet.bitcoin_credentials(config_file.bitcoin_network, 0)?;

		Ok(Self {
			state_directory,
			stacks_credentials,
			bitcoin_credentials,
			stacks_node_url,
			bitcoin_node_url,
			contract_name: ContractName::from(
				config_file.contract_name.as_str(),
			),
		})
	}
}

fn normalize(root_dir: PathBuf, path: impl AsRef<Path>) -> PathBuf {
	if path.as_ref().is_relative() {
		root_dir.join(path)
	} else {
		path.as_ref().into()
	}
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ConfigFile {
	/// Directory to persist the state of the system to
	pub state_directory: PathBuf,

	/// Seed mnemonic
	pub mnemonic: String,

	/// Stacks network
	pub stacks_network: StacksNetwork,

	/// Bitcoin network
	pub bitcoin_network: BitcoinNetwork,

	/// Address of a stacks node
	pub stacks_node_url: String,

	/// Address of a bitcoin node
	pub bitcoin_node_url: String,

	/// sBTC asset contract name
	pub contract_name: String,
}

impl ConfigFile {
	pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
		let config_file = File::open(&path)?;

		Ok(serde_json::from_reader(config_file)?)
	}
}

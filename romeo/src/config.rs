//! Config

use std::{
	fs::File,
	path::{Path, PathBuf},
};

use bdk::bitcoin::Network as BitcoinNetwork;
use blockstack_lib::vm::ContractName;
use clap::Parser;
use stacks_core::{
	wallet::{BitcoinCredentials, Credentials as StacksCredentials, Wallet},
	Network as StacksNetwork, StacksError,
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

	/// Stacks network
	pub stacks_network: StacksNetwork,

	/// Bitcoin network
	pub bitcoin_network: BitcoinNetwork,

	/// Credentials used to interact with the Stacks network
	pub stacks_credentials: StacksCredentials,
	/// Credentials used to interact with the Bitcoin network
	pub bitcoin_credentials: BitcoinCredentials,

	/// Address of a stacks node
	pub stacks_node_url: Url,

	/// Address of a bitcoin node
	pub bitcoin_node_url: Url,

	/// Address of the Electrum node
	pub electrum_node_url: Url,

	/// sBTC asset contract name
	pub contract_name: ContractName,

	/// optional api key used for the stacks node
	pub hiro_api_key: Option<String>,

	/// Strict mode
	pub strict: bool,
}

impl Config {
	/// Read the config file in the path
	pub fn try_from_path(path: impl AsRef<Path>) -> Result<Self, StacksError> {
		ConfigFile::try_from(path.as_ref()).unwrap().try_into()
	}

	/// The sbtc wallet address is the taproot address
	/// of the bitcoin credentials
	pub fn sbtc_wallet_address(&self) -> bdk::bitcoin::Address {
		self.bitcoin_credentials.address_p2tr()
	}
}

impl TryFrom<ConfigFile> for Config {
	type Error = stacks_core::StacksError;

	fn try_from(config_file: ConfigFile) -> Result<Self, Self::Error> {
		let wallet = Wallet::new(&config_file.mnemonic)?;

		let stacks_credentials =
			wallet.credentials(config_file.stacks_network, 0)?;
		let bitcoin_credentials =
			wallet.bitcoin_credentials(config_file.bitcoin_network, 0)?;
		let hiro_api_key = config_file.hiro_api_key;

		Ok(Self {
			state_directory: config_file.state_directory,
			stacks_network: config_file.stacks_network,
			bitcoin_network: config_file.bitcoin_network,
			stacks_credentials,
			bitcoin_credentials,
			stacks_node_url: config_file.stacks_node_url,
			bitcoin_node_url: config_file.bitcoin_node_url,
			electrum_node_url: config_file.electrum_node_url,
			contract_name: ContractName::from(
				config_file.contract_name.as_str(),
			),
			hiro_api_key,
			strict: config_file.strict,
		})
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
	pub stacks_node_url: Url,

	/// Address of a bitcoin node
	pub bitcoin_node_url: Url,

	/// Address of the Electrum node
	pub electrum_node_url: Url,

	/// sBTC asset contract name
	pub contract_name: ContractName,

	/// optional api key used for the stacks node
	pub hiro_api_key: Option<String>,

	/// Strict mode
	#[serde(default)]
	pub strict: bool,
}

impl TryFrom<&Path> for ConfigFile {
	type Error = std::io::Error;

	fn try_from(value: &Path) -> Result<Self, Self::Error> {
		let config_file = File::open(value)?;
		Ok(serde_json::from_reader(config_file)?)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn deserialize() {
		let path_str = "../devenv/sbtc/docker/config.json";
		// this file is now the ground truth.
		let path = Path::new(path_str);
		Config::try_from_path(path).unwrap();

		let path = path_str;
		Config::try_from_path(path).unwrap();
	}
}

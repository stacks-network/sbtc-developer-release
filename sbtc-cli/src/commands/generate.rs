use std::io::stdout;

use bdk::bitcoin::Network as BitcoinNetwork;
use clap::Parser;
use serde_json::{Map, Value};
use stacks_core::{
	wallet::{BitcoinCredentials, Credentials, Wallet},
	Network as StacksNetwork,
};

#[derive(Parser, Debug, Clone)]
pub struct GenerateArgs {
	/// Specify how to generate the credentials
	#[command(subcommand)]
	subcommand: GenerateSubcommand,

	/// Stacks network to generate credentials for
	#[clap(short, long, default_value_t = StacksNetwork::Mainnet)]
	stacks_network: StacksNetwork,

	/// Bitcoin network to generate credentials for
	#[clap(short, long, default_value_t = BitcoinNetwork::Bitcoin)]
	bitcoin_network: BitcoinNetwork,

	/// How many accounts to generate
	#[clap(short, long, default_value_t = 1)]
	accounts: usize,
}

#[derive(clap::Subcommand, Debug, Clone)]
enum GenerateSubcommand {
	New,
	Mnemonic { mnemonic: String },
}

pub fn generate(generate_args: &GenerateArgs) -> anyhow::Result<()> {
	match &generate_args.subcommand {
		GenerateSubcommand::New => {
			let wallet = Wallet::random()?;

			serde_json::to_writer_pretty(
				stdout(),
				&value_from_wallet(&wallet, generate_args),
			)?;
		}
		GenerateSubcommand::Mnemonic { mnemonic } => {
			let wallet = Wallet::new(mnemonic)?;

			serde_json::to_writer_pretty(
				stdout(),
				&value_from_wallet(&wallet, generate_args),
			)?;
		}
	};

	Ok(())
}

fn value_from_wallet(wallet: &Wallet, generate_args: &GenerateArgs) -> Value {
	let mut map = Map::new();

	map.insert("mnemonic".into(), wallet.mnemonic().to_string().into());
	map.insert(
		"private_key".into(),
		hex::encode(wallet.master_key().secret_bytes()).into(),
	);
	map.insert(
		"wif".into(),
		wallet.wif(generate_args.stacks_network).to_string().into(),
	);

	let mut credentials: Vec<Value> = Default::default();

	for i in 0..generate_args.accounts {
		let mut creds = Map::new();
		creds.insert(
			"stacks".into(),
			value_from_credentials(
				wallet
					.credentials(generate_args.stacks_network, i as u32)
					.unwrap(),
			),
		);
		creds.insert(
			"bitcoin".into(),
			value_from_bitcoin_credentials(
				wallet
					.bitcoin_credentials(
						generate_args.bitcoin_network,
						i as u32,
					)
					.unwrap(),
			),
		);

		credentials.push(creds.into());
	}

	map.insert(
		"credentials".into(),
		credentials
			.into_iter()
			.enumerate()
			.map(|(i, creds)| (i.to_string(), creds))
			.collect::<Map<String, Value>>()
			.into(),
	);

	map.insert(
		"network_stacks".into(),
		generate_args
			.stacks_network
			.to_string()
			.to_ascii_lowercase()
			.into(),
	);
	map.insert(
		"network_bitcoin".into(),
		generate_args
			.bitcoin_network
			.to_string()
			.to_ascii_lowercase()
			.into(),
	);

	map.into()
}

fn value_from_credentials(creds: Credentials) -> Value {
	let mut stacks_creds = Map::new();

	stacks_creds.insert(
		"private_key".into(),
		hex::encode(creds.private_key().secret_bytes()).into(),
	);
	stacks_creds
		.insert("public_key".into(), creds.public_key().to_string().into());
	stacks_creds.insert("address".into(), creds.address().to_string().into());
	stacks_creds.insert("wif".into(), creds.wif().to_string().into());

	stacks_creds.into()
}

fn value_from_bitcoin_credentials(creds: BitcoinCredentials) -> Value {
	let mut btc_creds = Map::new();

	let mut btc_p2pkh_creds = Map::new();
	btc_p2pkh_creds.insert(
		"private_key".into(),
		hex::encode(creds.private_key_p2pkh().secret_bytes()).into(),
	);
	btc_p2pkh_creds.insert(
		"public_key".into(),
		creds.public_key_p2pkh().to_string().into(),
	);
	btc_p2pkh_creds
		.insert("address".into(), creds.address_p2pkh().to_string().into());
	btc_p2pkh_creds.insert("wif".into(), creds.wif_p2pkh().to_string().into());
	btc_creds.insert("p2pkh".into(), btc_p2pkh_creds.into());

	let mut btc_p2wpkh_creds = Map::new();
	btc_p2wpkh_creds.insert(
		"private_key".into(),
		hex::encode(creds.private_key_p2wpkh().secret_bytes()).into(),
	);
	btc_p2wpkh_creds.insert(
		"public_key".into(),
		creds.public_key_p2wpkh().to_string().into(),
	);
	btc_p2wpkh_creds
		.insert("address".into(), creds.address_p2wpkh().to_string().into());
	btc_p2wpkh_creds
		.insert("wif".into(), creds.wif_p2wpkh().to_string().into());
	btc_creds.insert("p2wpkh".into(), btc_p2wpkh_creds.into());

	let mut btc_p2tr_creds = Map::new();
	btc_p2tr_creds.insert(
		"private_key".into(),
		hex::encode(creds.private_key_p2tr().secret_bytes()).into(),
	);
	btc_p2tr_creds.insert(
		"public_key".into(),
		creds.public_key_p2tr().to_string().into(),
	);
	btc_p2tr_creds
		.insert("address".into(), creds.address_p2tr().to_string().into());
	btc_p2tr_creds.insert("wif".into(), creds.wif_p2tr().to_string().into());
	btc_creds.insert("p2tr".into(), btc_p2tr_creds.into());

	btc_creds.into()
}

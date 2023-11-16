use std::{io::Cursor, time::Duration};

use bdk::{
	bitcoin::{psbt::serialize::Serialize, PrivateKey},
	blockchain::{
		ConfigurableBlockchain, ElectrumBlockchain, ElectrumBlockchainConfig,
	},
	database::MemoryDatabase,
	template::P2Wpkh,
	SyncOptions, Wallet,
};
use blockstack_lib::{
	codec::StacksMessageCodec,
	util::hash::hex_bytes,
	vm::{
		types::{QualifiedContractIdentifier, StandardPrincipalData},
		Value,
	},
};
use romeo::{config::Config, stacks_client::StacksClient};
use sbtc_cli::commands::{
	broadcast::{broadcast_tx, BroadcastArgs},
	deposit::{build_deposit_tx, DepositArgs},
	withdraw::{build_withdrawal_tx, WithdrawalArgs},
};
use stacks_core::address::StacksAddress;
use tokio::time::sleep;
use url::Url;

/// Wait until all your services are ready before running.
/// Don't forget to fund W0 (deployer) and W1 (recipient).
#[tokio::main]
async fn main() {
	let mut config =
		Config::from_path("./devenv/sbtc/docker/config.json").unwrap();
	config.stacks_node_url = "http://localhost:3999".parse().unwrap();
	config.bitcoin_node_url = "http://localhost:18443".parse().unwrap();
	config.electrum_node_url = "tcp://localhost:60401".parse().unwrap();

	let blockchain =
		ElectrumBlockchain::from_config(&ElectrumBlockchainConfig {
			url: config.electrum_node_url.clone().into(),
			socks5: None,
			retry: 3,
			timeout: Some(10),
			stop_gap: 10,
			validate_domain: false,
		})
		.unwrap();

	let recipient_p2wpkh_wif =
		"cNcXK2r8bNdWJQymtAW8tGS7QHNtFFvG5CdXqhhT752u29WspXRM";

	// W1
	let wallet = {
		let private_key = PrivateKey::from_wif(recipient_p2wpkh_wif).unwrap();

		Wallet::new(
			P2Wpkh(private_key),
			Some(P2Wpkh(private_key)),
			bdk::bitcoin::Network::Regtest,
			MemoryDatabase::default(),
		)
		.unwrap()
	};

	loop {
		wallet.sync(&blockchain, SyncOptions::default()).unwrap();
		let balance = wallet.get_balance().unwrap().confirmed;
		println!("recipient's btc: {balance}");
		if balance != 0 {
			break;
		}
		sleep(Duration::from_secs(1)).await;
	}

	let recipient = "ST2ST2H80NP5C9SPR4ENJ1Z9CDM9PKAJVPYWPQZ50";
	let amount = 1000;

	// deposit
	{
		let electrum_url =
			Url::parse(config.electrum_node_url.as_str()).unwrap();
		let tx = {
			let args = DepositArgs {
			node_url: electrum_url.clone(),
			wif: recipient_p2wpkh_wif.into(),
			network: bdk::bitcoin::Network::Regtest,
            recipient:recipient.into(),
			amount,
			sbtc_wallet: "bcrt1pte5zmd7qzj4hdu45lh9mmdm0nwq3z35pwnxmzkwld6y0a8g83nnqhj6vc0".into(),
		};

			build_deposit_tx(&args).unwrap()
		};

		broadcast_tx(&BroadcastArgs {
			node_url: electrum_url,
			tx: hex::encode(tx.serialize()),
		})
		.unwrap();
	}

	let deployer = "ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM";
	let deployer_address = StacksAddress::transmute_stacks_address(deployer);
	let recipient_address = StacksAddress::transmute_stacks_address(recipient);

	let stacks_client =
		StacksClient::new(config.clone(), reqwest::Client::new());

	println!("Waiting on sBTC mint");
	// request token balance from the asset contract.
	while {
		let res: serde_json::Value = stacks_client
			.call_read_only_fn(
				QualifiedContractIdentifier::new(
					StandardPrincipalData::from(deployer_address),
					config.contract_name.clone(),
				),
				"get-balance",
				recipient_address.to_string().as_str(),
				vec![StandardPrincipalData::from(recipient_address).into()],
			)
			.await
			.unwrap();

		assert!(res["okay"].as_bool().unwrap());
		let bytes =
			hex_bytes(res["result"].as_str().unwrap().trim_start_matches("0x"))
				.unwrap();

		let mut cursor = Cursor::new(&bytes);
		Value::consensus_deserialize(&mut cursor)
			.unwrap()
			.expect_result_ok()
			.expect_u128()
	} < amount as u128
	{
		sleep(Duration::from_secs(2)).await;
	}

	let fee = 331;
	// withdraw
	let args = WithdrawalArgs {
		node_url: config.electrum_node_url.clone(),
		network: bdk::bitcoin::Network::Regtest,
		// p2wpkh
		wif: "cNcXK2r8bNdWJQymtAW8tGS7QHNtFFvG5CdXqhhT752u29WspXRM".into(),
		// Stacks
		drawee_wif: "cR9hENRFiuHzKpj9B3QCTBrt19c5ZCJKHJwYcqj5dfB6aKyf6ndm"
			.into(),
		payee_address: "bcrt1q3zl64vadtuh3vnsuhdgv6pm93n82ye8q6cr4ch".into(),
		amount,
		fulfillment_fee: fee,
		sbtc_wallet:
			"bcrt1pte5zmd7qzj4hdu45lh9mmdm0nwq3z35pwnxmzkwld6y0a8g83nnqhj6vc0"
				.into(),
	};

	let tx = build_withdrawal_tx(&args).unwrap();

	let balance = loop {
		wallet.sync(&blockchain, SyncOptions::default()).unwrap();
		let balance = wallet.get_balance().unwrap().confirmed;
		if 0 < balance {
			println!("recipient's btc: {balance}");
			break balance;
		}
		sleep(Duration::from_secs(1)).await
	};

	broadcast_tx(&BroadcastArgs {
		node_url: config.electrum_node_url.clone(),
		tx: hex::encode(tx.serialize()),
	})
	.unwrap();

	println!("Waiting on fulfillment");
	loop {
		wallet.sync(&blockchain, SyncOptions::default()).unwrap();
		let current = wallet.get_balance().unwrap().confirmed;
		// will fail if tx_fees is not an upper bound for real fees.
		let tx_fees = 400;
		println!("recipient's btc: {balance}");
		if balance.saturating_sub(fee + tx_fees) < current {
			break;
		}
		sleep(Duration::from_secs(2)).await;
	}
}

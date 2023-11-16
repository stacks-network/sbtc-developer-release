use std::{str::FromStr, time::Duration};

use bdk::{
	bitcoin::{psbt::serialize::Serialize, Address, PrivateKey},
	bitcoincore_rpc::RpcApi,
	blockchain::{
		ConfigurableBlockchain, ElectrumBlockchain, ElectrumBlockchainConfig,
	},
	database::MemoryDatabase,
	template::P2Wpkh,
	SyncOptions, Wallet,
};
use romeo::{config::Config, stacks_client::StacksClient};
use sbtc_cli::commands::{
	broadcast::{broadcast_tx, BroadcastArgs},
	withdraw::{build_withdrawal_tx, WithdrawalArgs},
};
use stacks_core::address::StacksAddress;
use tokio::time::sleep;

use super::{
	bitcoin_client::{
		bitcoin_url, client_new, electrs_url, sbtc_balance,
		wait_for_tx_confirmation,
	},
	KeyType::*,
	WALLETS,
};

#[tokio::test]
/// Depends on deposit test.
async fn broadcast_withdrawal() {
	// wait until stacks addr has some balance

	let deployer_address =
		StacksAddress::transmute_stacks_address(WALLETS[0][Stacks].address);
	let recipient_address =
		StacksAddress::transmute_stacks_address(WALLETS[1][Stacks].address);

	let config = Config::from_path("config.json").unwrap();

	let stacks_client =
		StacksClient::new(config.clone(), reqwest::Client::new());

	// sbtc credited
	let amount = loop {
		let amount = sbtc_balance(
			&stacks_client,
			deployer_address,
			recipient_address,
			config.contract_name.clone(),
		)
		.await;
		if amount != 0 {
			break amount as u64;
		}
		sleep(Duration::from_secs(2)).await;
	};

	let b_client = client_new(bitcoin_url().as_str(), "devnet", "devnet");

	b_client
		.import_address(
			&Address::from_str(WALLETS[1][P2wpkh].address).unwrap(),
			None,
			Some(false),
		)
		.unwrap();

	let args = WithdrawalArgs {
		node_url: electrs_url(),
		network: bdk::bitcoin::Network::Regtest,
		wif: WALLETS[1][P2wpkh].wif.into(),
		drawee_wif: WALLETS[1][Stacks].wif.into(),
		payee_address: WALLETS[2][P2wpkh].address.into(),
		amount,
		fulfillment_fee: 2000,
		sbtc_wallet: WALLETS[0][P2tr].address.into(),
	};

	let tx = build_withdrawal_tx(&args).unwrap();

	broadcast_tx(&BroadcastArgs {
		node_url: electrs_url(),
		tx: hex::encode(tx.serialize()),
	})
	.unwrap();

	let txid = tx.txid();

	wait_for_tx_confirmation(&b_client, &txid, 1).await;

	// sbtc debited
	{
		while {
			sbtc_balance(
				&stacks_client,
				deployer_address,
				recipient_address,
				config.contract_name.clone(),
			)
		}
		.await != 0
		{
			sleep(Duration::from_secs(2)).await;
		}
	}

	// btc credited
	{
		let blockchain =
			ElectrumBlockchain::from_config(&ElectrumBlockchainConfig {
				url: electrs_url().into(),
				socks5: None,
				retry: 3,
				timeout: Some(10),
				stop_gap: 10,
				validate_domain: false,
			})
			.unwrap();

		let private_key = PrivateKey::from_wif(WALLETS[2][P2wpkh].wif).unwrap();

		let wallet = Wallet::new(
			P2Wpkh(private_key),
			Some(P2Wpkh(private_key)),
			bdk::bitcoin::Network::Regtest,
			MemoryDatabase::default(),
		)
		.unwrap();

		while {
			wallet.sync(&blockchain, SyncOptions::default()).unwrap();
			wallet.get_balance().unwrap().confirmed
		} == 0
		{
			sleep(Duration::from_secs(2)).await;
		}
	}
}

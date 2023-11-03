use std::{thread::sleep, time::Duration};

use anyhow::Result;
use bdk::{
	bitcoin::{psbt::serialize::Serialize, PrivateKey},
	blockchain::{
		ConfigurableBlockchain, ElectrumBlockchain, ElectrumBlockchainConfig,
	},
	database::MemoryDatabase,
	template::P2Wpkh,
	SyncOptions, Wallet,
};
use reqwest::blocking::Client;
use sbtc_cli::commands::{
	broadcast::{broadcast_tx, BroadcastArgs},
	deposit::{build_deposit_tx, DepositArgs},
};

use super::{
	bitcoin_client::{electrs_url, generate_blocks},
	KeyType::*,
	WALLETS,
};

#[test]
fn broadcast_deposit() -> Result<()> {
	let client = Client::new();
	{
		generate_blocks(1, &client, WALLETS[0][P2wpkh].address);
		generate_blocks(1, &client, WALLETS[1][P2wpkh].address);
		// pads blocks to get rewards.
		generate_blocks(100, &client, WALLETS[0][P2wpkh].address);
	};

	let electrum_url = electrs_url();

	// suboptimal, replace once we have better events.
	{
		let blockchain =
			ElectrumBlockchain::from_config(&ElectrumBlockchainConfig {
				url: electrum_url.clone().into(),
				socks5: None,
				retry: 3,
				timeout: Some(10),
				stop_gap: 10,
				validate_domain: false,
			})
			.unwrap();

		let private_key = PrivateKey::from_wif(WALLETS[1][P2wpkh].wif)?;

		let wallet = Wallet::new(
			P2Wpkh(private_key),
			Some(P2Wpkh(private_key)),
			bdk::bitcoin::Network::Regtest,
			MemoryDatabase::default(),
		)
		.unwrap();

		loop {
			wallet.sync(&blockchain, SyncOptions::default()).unwrap();
			let balance = wallet.get_balance().unwrap();
			if balance.confirmed != 0 {
				break;
			}
			sleep(Duration::from_millis(1_000));
		}
	}

	let amount = 10_000;

	let tx = {
		let args = DepositArgs {
			node_url: electrum_url.clone(),
			wif: WALLETS[1][P2wpkh].wif.into(),
			network: bdk::bitcoin::Network::Regtest,
			recipient: WALLETS[1][Stacks].address.into(),
			amount,
			sbtc_wallet: WALLETS[0][P2tr].address.into(),
		};

		build_deposit_tx(&args).unwrap()
	};

	broadcast_tx(&BroadcastArgs {
		node_url: electrum_url,
		tx: hex::encode(tx.serialize()),
	})
	.unwrap();

	Ok(())
}

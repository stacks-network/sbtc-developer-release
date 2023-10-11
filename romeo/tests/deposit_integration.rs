use std::{thread::sleep, time::Duration};

use anyhow::Result;
use bdk::bitcoin::psbt::serialize::Serialize;
use reqwest::blocking::Client;
use sbtc_cli::commands::{
	broadcast::{broadcast_tx, BroadcastArgs},
	deposit::{build_deposit_tx, DepositArgs},
};

mod bitcoin_client_integration;

use bitcoin_client_integration::{electrs_url, generate_blocks};

const WALLET_0_P2TR_ADDRESS: &str =
	"bcrt1pte5zmd7qzj4hdu45lh9mmdm0nwq3z35pwnxmzkwld6y0a8g83nnqhj6vc0";
const WALLET_0_P2WPKH_ADDRESS: &str =
	"bcrt1q3zl64vadtuh3vnsuhdgv6pm93n82ye8q6cr4ch";
const WALLET_1_P2WPKH_WIF: &str =
	"cNcXK2r8bNdWJQymtAW8tGS7QHNtFFvG5CdXqhhT752u29WspXRM";
const WALLET_1_STX_ADDRESS: &str = "ST2ST2H80NP5C9SPR4ENJ1Z9CDM9PKAJVPYWPQZ50";
const WALLET_1_P2WPKH_ADDRESS: &str =
	"bcrt1q3tj2fr9scwmcw3rq5m6jslva65f2rqjxfrjz47";

use bdk::{
	bitcoin::PrivateKey,
	blockchain::{
		ConfigurableBlockchain, ElectrumBlockchain, ElectrumBlockchainConfig,
	},
	database::MemoryDatabase,
	template::P2Wpkh,
	SyncOptions, Wallet,
};

#[test]
fn broadcast_deposit() -> Result<()> {
	let client = Client::new();
	{
		generate_blocks(1, &client, WALLET_1_P2WPKH_ADDRESS);
		generate_blocks(1, &client, WALLET_0_P2WPKH_ADDRESS);
		// pads blocks to get rewards.
		generate_blocks(200, &client, WALLET_1_P2WPKH_ADDRESS);
	};

	let electrum_url = electrs_url();
	let amount = 10_000;

	// suboptimal, replace once we have better events.
	{
		let private_key = PrivateKey::from_wif(WALLET_1_P2WPKH_WIF)?;

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

	let tx = {
		let args = DepositArgs {
			node_url: electrum_url.clone(),
			wif: WALLET_1_P2WPKH_WIF.into(),
			network: bdk::bitcoin::Network::Regtest,
			recipient: WALLET_1_STX_ADDRESS.into(),
			amount,
			sbtc_wallet: WALLET_0_P2TR_ADDRESS.into(),
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

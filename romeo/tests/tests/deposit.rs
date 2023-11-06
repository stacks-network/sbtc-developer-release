use std::{str::FromStr, thread::sleep, time::Duration};

use anyhow::Result;
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
use sbtc_cli::commands::{
	broadcast::{broadcast_tx, BroadcastArgs},
	deposit::{build_deposit_tx, DepositArgs},
};

use super::{
	bitcoin_client::{
		bitcoin_url, client_new, electrs_url, mine_blocks,
		wait_for_tx_confirmation,
	},
	KeyType::*,
	WALLETS,
};

#[test]
fn broadcast_deposit() -> Result<()> {
	let b_client = client_new(bitcoin_url().as_str(), "devnet", "devnet");

	b_client
		.import_address(
			&Address::from_str(WALLETS[1][P2wpkh].address).unwrap(),
			None,
			Some(false),
		)
		.unwrap();

	{
		mine_blocks(&b_client, 1, WALLETS[0][P2wpkh].address);
		mine_blocks(&b_client, 1, WALLETS[1][P2wpkh].address);
		// pads blocks to get rewards.
		mine_blocks(&b_client, 100, WALLETS[0][P2wpkh].address);
	};

	let electrum_url = electrs_url();

	// suboptimal, replace once we have better events.
	// replace with b_client balance?
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

	let txid = tx.txid();

	wait_for_tx_confirmation(&b_client, &txid, 1);

	Ok(())
}

use std::{str::FromStr, thread::sleep, time::Duration};

use bdk::{
	bitcoin::{hash_types::Txid, Address, BlockHash},
	bitcoincore_rpc::{Auth, Client as BClient, RpcApi},
};
use url::Url;

pub fn bitcoin_url() -> Url {
	Url::parse("http://bitcoin:18443").unwrap()
}

pub fn electrs_url() -> Url {
	Url::parse("tcp://electrs:60401").unwrap()
}

pub fn client_new(url: &str, user: &str, pass: &str) -> BClient {
	BClient::new(url, Auth::UserPass(user.into(), pass.into())).unwrap()
}

pub fn mine_blocks(
	client: &BClient,
	blocks: u64,
	address: &str,
) -> Vec<BlockHash> {
	client
		.generate_to_address(blocks, &Address::from_str(address).unwrap())
		.unwrap()
}

pub fn wait_for_tx_confirmation(
	b_client: &BClient,
	txid: &Txid,
	confirmations: i32,
) {
	loop {
		match b_client.get_transaction(txid, None) {
			Ok(tx) if tx.info.confirmations >= confirmations => {
				break;
			}
			Ok(ok) => {
				println!("Waiting confirmation on {txid}:{ok:?}")
			}
			Err(e) => {
				println!("Waiting confirmation on {txid}:{e:?}")
			}
		}

		sleep(Duration::from_secs(1));
	}
}

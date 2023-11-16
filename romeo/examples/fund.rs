use std::str::FromStr;

use bdk::{
	bitcoin::{Address, BlockHash},
	bitcoincore_rpc::{Auth, Client, RpcApi},
};

fn mine_blocks(client: &Client, blocks: u64, address: &str) -> Vec<BlockHash> {
	client
		.generate_to_address(blocks, &Address::from_str(address).unwrap())
		.unwrap()
}

fn main() {
	let client = Client::new(
		"http://localhost:18443",
		Auth::UserPass("devnet".into(), "devnet".into()),
	)
	.unwrap();

	// p2wpkh W0
	let address_0 = "bcrt1q3tj2fr9scwmcw3rq5m6jslva65f2rqjxfrjz47";
	let block_hashes = mine_blocks(&client, 10, address_0);
	println!("blocks mined: {block_hashes:#?}");
	let address_1 = "bcrt1q3zl64vadtuh3vnsuhdgv6pm93n82ye8q6cr4ch";
	let block_hashes = mine_blocks(&client, 10, address_1);
	println!("blocks mined: {block_hashes:#?}");
	let address_0 = "bcrt1q3tj2fr9scwmcw3rq5m6jslva65f2rqjxfrjz47";
	let block_hashes = mine_blocks(&client, 101, address_0);
	println!("padding blocks mined: {block_hashes:#?}");
}

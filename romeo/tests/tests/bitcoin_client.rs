use std::{io::Cursor, str::FromStr, thread::sleep, time::Duration};

use bdk::{
	bitcoin::{hash_types::Txid, Address, BlockHash},
	bitcoincore_rpc::{Auth, Client as BClient, RpcApi},
};
use blockstack_lib::{
	codec::StacksMessageCodec,
	types::chainstate::StacksAddress,
	util::hash::hex_bytes,
	vm::{
		types::{QualifiedContractIdentifier, StandardPrincipalData},
		ContractName, Value,
	},
};
use romeo::stacks_client::StacksClient;
use url::Url;

/// devenv's service url
pub fn bitcoin_url() -> Url {
	Url::parse("http://bitcoin:18443").unwrap()
}

/// devenv's service url
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

pub async fn sbtc_balance(
	stacks_client: &StacksClient,
	deployer_address: StacksAddress,
	recipient_address: StacksAddress,
	contract_name: ContractName,
) -> u128 {
	let res: serde_json::Value = stacks_client
		.call_read_only_fn(
			QualifiedContractIdentifier::new(
				StandardPrincipalData::from(deployer_address),
				contract_name,
			),
			"get-balance",
			recipient_address.to_string().as_str(),
			vec![StandardPrincipalData::from(recipient_address).into()],
		)
		.await
		.unwrap();

	assert!(res["okay"].as_bool().unwrap());
	// request token balance from the asset contract.
	let bytes =
		hex_bytes(res["result"].as_str().unwrap().trim_start_matches("0x"))
			.unwrap();

	let mut cursor = Cursor::new(&bytes);
	Value::consensus_deserialize(&mut cursor)
		.unwrap()
		.expect_result_ok()
		.expect_u128()
}

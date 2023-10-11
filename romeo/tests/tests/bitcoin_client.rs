use std::env;

use reqwest::blocking::Client;
use serde_json::json;
use url::Url;

pub fn project_name() -> String {
	env::var("PROJECT_NAME").unwrap()
}

pub fn bitcoin_url() -> Url {
	let base = project_name();
	Url::parse(&format!("http://{base}-bitcoin-1:18443")).unwrap()
}

pub fn electrs_url() -> Url {
	let base = project_name();
	Url::parse(&format!("tcp://{base}-electrs-1:60401")).unwrap()
}

pub fn generate_blocks(
	blocks: u64,
	ctx: &Client,
	address: &str,
) -> Vec<String> {
	let endpoint = bitcoin_url();
	let user = "devnet";
	let password = "devnet";
	let body = json!({
		"jsonrpc": "1.0",
		"id": "1",
		"method": "generatetoaddress",
		//developer's
		"params": [blocks,address]
	});

	let response_json: serde_json::Value = ctx
		.post(endpoint)
		.basic_auth(user, Some(password))
		.header(reqwest::header::CONTENT_TYPE, "application/json")
		.json(&body)
		.send()
		.unwrap()
		.json()
		.unwrap();

	assert_eq!(response_json["error"], serde_json::Value::Null);
	serde_json::from_value(response_json["result"].clone()).expect("block_ids")
}

#[test]
fn mine_empty_block() {
	let client = Client::new();
	generate_blocks(10, &client, "mqVnk6NPRdhntvfm4hh9vvjiRkFDUuSYsH");
}

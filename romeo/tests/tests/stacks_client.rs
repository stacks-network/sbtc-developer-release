use std::env;

use reqwest::blocking::Client;
use url::Url;

pub fn stacks_url() -> Url {
	let base = env::var("PROJECT_NAME").unwrap();
	Url::parse(&format!("http://{base}-stacks-1:20443")).unwrap()
}

pub fn fetch_stacks_height(ctx: &Client) -> u64 {
	let endpoint = stacks_url().join("v2/info").unwrap();

	let response_json: serde_json::Value =
		ctx.get(endpoint).send().unwrap().json().unwrap();

	serde_json::from_value(response_json["stacks_tip_height"].clone()).unwrap()
}

// Test fetch stacks info.
#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn fetch_height() {
		let ctx = Client::new();
		fetch_stacks_height(&ctx);
	}
}

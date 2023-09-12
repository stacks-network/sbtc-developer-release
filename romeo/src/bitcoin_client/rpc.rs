//! RPC Bitcoin client

use bdk::bitcoincore_rpc::{Auth, Client};
use url::Url;

pub struct RPCClient(Client);

impl RPCClient {
    pub fn new(mut url: Url) -> anyhow::Result<Self> {
        let username = url.username().to_string();

        if username.is_empty() {
            return Err(anyhow::anyhow!("Username is empty"));
        }

        let password = url.password().unwrap_or_default().to_string();

        url.set_username("").unwrap();
        url.set_password(None).unwrap();

        let client = Client::new(&url.to_string(), Auth::UserPass(username, password))?;

        Ok(Self(client))
    }
}

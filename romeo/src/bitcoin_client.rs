//! Bitcoin client

//use bdk::esplora_client::r#async::AsyncClient;

/// Stateless wrapper around a bdk esplora client
pub struct BitcoinClient {
    client: bdk::esplora_client::AsyncClient,
}

//! Bitcoin client trait

use async_trait::async_trait;
use bdk::bitcoin;

use crate::event;

#[async_trait]
/// Bitcoin client trait
pub trait BitcoinClient {
    /// Get the status of a transaction
    async fn get_tx_status(
        &mut self,
        txid: &bitcoin::Txid,
    ) -> anyhow::Result<event::TransactionStatus>;

    /// Get a bitcoin block by height
    async fn fetch_block(&mut self, block_height: u32) -> anyhow::Result<bitcoin::Block>;

    /// Get the current height of the Bitcoin chain
    async fn get_height(&mut self) -> anyhow::Result<u32>;
}

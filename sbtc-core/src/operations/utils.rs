//! Utilities for sBTC transactions

use bdk::{
	bitcoin::PrivateKey, blockchain::ElectrumBlockchain,
	database::MemoryDatabase, electrum_client::Client, template::P2Wpkh,
	SyncOptions, Wallet,
};

use crate::{SBTCError, SBTCResult};

/// Initializes the electrum blockchain client
pub(crate) fn init_blockchain() -> SBTCResult<ElectrumBlockchain> {
	let client = Client::new("ssl://blockstream.info:993").map_err(|err| {
		SBTCError::ElectrumError("Could not create Electrum client", err)
	})?;
	let blockchain = ElectrumBlockchain::from(client);

	Ok(blockchain)
}

/// Set up an electrum wallet for sBTC operations
pub(crate) fn setup_wallet(
	private_key: PrivateKey,
) -> SBTCResult<Wallet<MemoryDatabase>> {
	let blockchain = init_blockchain()?;

	let wallet = Wallet::new(
		P2Wpkh(private_key),
		Some(P2Wpkh(private_key)),
		private_key.network,
		MemoryDatabase::default(),
	)
	.map_err(|err| SBTCError::BDKError("Could not open wallet", err))?;

	wallet
		.sync(&blockchain, SyncOptions::default())
		.map_err(|err| SBTCError::BDKError("Could not sync wallet", err))?;

	Ok(wallet)
}

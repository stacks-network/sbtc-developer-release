use bdk::bitcoin::{secp256k1::PublicKey, Address as BitcoinAddress};
use stacks_core::address::StacksAddress;

#[derive(Clone, Debug)]
/// Configuration for the signer approval/denial
pub struct Config {
	/// The maximum dollar amount of a transaction that will be auto approved
	pub auto_approve_max_amount: u64,
	/// The public key of the signer being delegated to
	pub delegate_public_key: PublicKey,
	/// The BTC addresses to be auto denied
	pub auto_deny_addresses_btc: Vec<BitcoinAddress>,
	/// The STX addresses to be auto denied
	pub auto_deny_addresses_stx: Vec<StacksAddress>,
}

//! WIF parsing and construction of Stacks private keys.

use bdk::bitcoin::util::base58;
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

use super::Hashing;
use crate::{
	crypto::{sha256::DoubleSha256Hasher, PrivateKey},
	Network, StacksError, StacksResult,
};

/// Contains validated WIF bytes. It assumess compression is always used.
pub struct WIF([u8; WIF_LENGTH]);

impl WIF {
	/// Constructs a WIF from a network and private key
	pub fn new(network: Network, private_key: PrivateKey) -> Self {
		let mut bytes = [0u8; WIF_LENGTH];
		bytes[0] = WIFPrefix::from(network) as u8;
		bytes[1..33].copy_from_slice(&private_key.secret_bytes());
		bytes[33] = 0x01;

		let bytes_to_hash = bytes[..34].to_vec();
		bytes[34..].copy_from_slice(
			&DoubleSha256Hasher::new(bytes_to_hash).as_bytes()[..4],
		);

		Self(bytes)
	}

	/// Attempts to parse a WIF from a byte slice
	pub fn from_bytes(bytes: impl AsRef<[u8]>) -> StacksResult<Self> {
		let bytes: [u8; WIF_LENGTH] = bytes.as_ref().try_into()?;

		let wif = Self(bytes);
		wif.validate()?;

		Ok(wif)
	}

	fn validate(&self) -> StacksResult<()> {
		let valid_network_byte =
			WIFPrefix::iter().any(|prefix| prefix as u8 == self.0[0]);
		let valid_private_key = PrivateKey::from_slice(&self.0[1..33]).is_ok();
		let valid_compression_byte = self.0[33] == 0x01;
		let valid_checksum = DoubleSha256Hasher::new(&self.0[..34]).as_ref()
			[..4] == self.0[34..];

		if valid_network_byte
			&& valid_private_key
			&& valid_compression_byte
			&& valid_checksum
		{
			Ok(())
		} else {
			Err(StacksError::InvalidData("WIF is invalid".into()))
		}
	}

	/// Returns the network
	pub fn network(&self) -> StacksResult<Network> {
		match WIFPrefix::from_repr(self.0[0]) {
			Some(WIFPrefix::Mainnet) => Ok(Network::Mainnet),
			Some(WIFPrefix::Testnet) => Ok(Network::Testnet),
			_ => Err(StacksError::InvalidData("Unknown network byte".into())),
		}
	}

	/// Returns the private key
	pub fn private_key(&self) -> StacksResult<PrivateKey> {
		Ok(PrivateKey::from_slice(&self.0[1..33])?)
	}
}

/// WIF length consists of:
///
/// 1. [WIFPrefix] byte
/// 2. Private key bytes (32 bytes)
/// 3. Compression byte
/// 4. Checksum bytes ( 4 bytes)
pub const WIF_LENGTH: usize = 38;

/// WIF network prefix byte
#[derive(Debug, Clone, Copy, Display, PartialEq, Eq, EnumIter, FromRepr)]
#[repr(u8)]
pub enum WIFPrefix {
	/// Mainnet
	Mainnet = 128,
	/// Testnet
	Testnet = 239,
}

impl From<Network> for WIFPrefix {
	fn from(value: Network) -> Self {
		match value {
			Network::Mainnet => Self::Mainnet,
			Network::Testnet => Self::Testnet,
		}
	}
}

impl TryFrom<String> for WIF {
	type Error = StacksError;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		let wif = Self::from_bytes(base58::from(&value)?)?;
		wif.validate()?;

		Ok(wif)
	}
}

impl ToString for WIF {
	fn to_string(&self) -> String {
		base58::encode_slice(&self.0)
	}
}

#[cfg(test)]
mod tests {

	use bdk::bitcoin::secp256k1::Secp256k1;
	use rand::thread_rng;

	use super::*;

	#[test]
	fn wif() {
		let pk = Secp256k1::new().generate_keypair(&mut thread_rng()).0;

		for network in Network::iter() {
			let wif = WIF::new(network, pk);

			assert_eq!(wif.network().unwrap(), network);
			assert_eq!(wif.private_key().unwrap(), pk);

			let bitcoin_pk =
				bdk::bitcoin::PrivateKey::from_wif(&wif.to_string()).unwrap();

			assert_eq!(pk.secret_bytes().as_slice(), &bitcoin_pk.to_bytes());
			assert_eq!(wif.to_string(), bitcoin_pk.to_wif());
			assert_eq!(bitcoin_pk.network, network.into());
		}
	}
}

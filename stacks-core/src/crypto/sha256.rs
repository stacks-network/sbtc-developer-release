pub use bdk::bitcoin::secp256k1;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
	crypto::{Hasher, Hashing, Hex},
	StacksError, StacksResult,
};

pub(crate) const SHA256_LENGTH: usize = 32;

#[derive(
	Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(try_from = "Hex")]
#[serde(into = "Hex")]
/// The Sha256 hashing type
pub struct Sha256Hashing([u8; SHA256_LENGTH]);

impl Hashing<SHA256_LENGTH> for Sha256Hashing {
	fn hash(data: &[u8]) -> Self {
		Self(Sha256::digest(data).into())
	}

	fn as_bytes(&self) -> &[u8] {
		&self.0
	}

	fn from_bytes(bytes: &[u8]) -> StacksResult<Self> {
		Ok(Self(bytes.try_into()?))
	}
}

// From conversion is fallible for this type
#[allow(clippy::from_over_into)]
impl Into<Hex> for Sha256Hashing {
	fn into(self) -> Hex {
		Hex(hex::encode(self.as_bytes()))
	}
}

impl TryFrom<Hex> for Sha256Hashing {
	type Error = StacksError;

	fn try_from(value: Hex) -> Result<Self, Self::Error> {
		Self::from_bytes(&hex::decode(value.0)?)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// The DoubleSha256 hashing type
pub struct DoubleSha256Hashing(Sha256Hashing);

impl Hashing<SHA256_LENGTH> for DoubleSha256Hashing {
	fn hash(data: &[u8]) -> Self {
		Self(Sha256Hashing::hash(Sha256Hashing::hash(data).as_bytes()))
	}

	fn as_bytes(&self) -> &[u8] {
		self.0.as_bytes()
	}

	fn from_bytes(bytes: &[u8]) -> StacksResult<Self> {
		Ok(Self(Sha256Hashing::from_bytes(bytes)?))
	}
}

// From conversion is fallible for this type
#[allow(clippy::from_over_into)]
impl Into<Hex> for DoubleSha256Hashing {
	fn into(self) -> Hex {
		Hex(hex::encode(self.as_bytes()))
	}
}

impl TryFrom<Hex> for DoubleSha256Hashing {
	type Error = StacksError;

	fn try_from(value: Hex) -> Result<Self, Self::Error> {
		Self::from_bytes(&hex::decode(value.0)?)
	}
}

/// The Sha256 hasher type
pub type Sha256Hasher = Hasher<Sha256Hashing, SHA256_LENGTH>;
/// The DoubleSha256 hasher type
pub type DoubleSha256Hasher = Hasher<DoubleSha256Hashing, SHA256_LENGTH>;

#[cfg(test)]
mod tests {
	use super::*;
	use crate::uint::Uint256;

	#[test]
	fn should_sha256_hash_correctly() {
		let plaintext = "Hello world";
		let expected_hash_hex =
			"64ec88ca00b268e5ba1a35678a1b5316d212f4f366b2477232534a8aeca37f3c";

		assert_eq!(
			hex::encode(Sha256Hasher::hash(plaintext.as_bytes())),
			expected_hash_hex
		);
	}

	#[test]
	fn should_sha256_checksum_correctly() {
		let plaintext = "Hello world";
		let expected_checksum_hex = "64ec88ca";

		assert_eq!(
			hex::encode(Sha256Hasher::hash(plaintext.as_bytes()).checksum()),
			expected_checksum_hex
		);
	}

	#[test]
	fn should_double_sha256_hash_correctly() {
		let plaintext = "Hello world";
		let expected_hash_hex =
			"f6dc724d119649460e47ce719139e521e082be8a9755c5bece181de046ee65fe";

		assert_eq!(
			hex::encode(
				DoubleSha256Hasher::hash(plaintext.as_bytes()).as_bytes()
			),
			expected_hash_hex
		);
	}

	#[test]
	fn should_double_sha256_checksum_correctly() {
		let plaintext = "Hello world";
		let expected_checksum_hex = "f6dc724d";

		assert_eq!(
			hex::encode(
				DoubleSha256Hasher::hash(plaintext.as_bytes()).checksum()
			),
			expected_checksum_hex
		);
	}

	#[test]
	fn should_convert_to_uint_correctly() {
		let expected_num = Uint256::from(0xDEADBEEFDEADBEEF as u64) << 64
			| Uint256::from(0x0102030405060708 as u64);
		let num_bytes = hex::decode(
			"0807060504030201efbeaddeefbeadde00000000000000000000000000000000",
		)
		.unwrap();

		let hash = Sha256Hashing(num_bytes.try_into().unwrap());

		assert_eq!(
			expected_num,
			Uint256::from_le_bytes(hash.as_bytes()).unwrap()
		);
	}
}

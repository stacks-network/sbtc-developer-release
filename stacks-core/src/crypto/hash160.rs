use ripemd::{Digest, Ripemd160};
use serde::{Deserialize, Serialize};

use super::sha256::Sha256Hasher;
use crate::{
	crypto::{Hasher, Hashing, Hex},
	StacksError, StacksResult,
};

pub(crate) const HASH160_LENGTH: usize = 20;

#[derive(
	Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(try_from = "Hex")]
#[serde(into = "Hex")]
/// Hash160 hash type
pub struct Hash160Hashing([u8; HASH160_LENGTH]);

impl Hashing<HASH160_LENGTH> for Hash160Hashing {
	fn hash(data: &[u8]) -> Self {
		Self(Ripemd160::digest(Sha256Hasher::new(data)).into())
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
impl Into<Hex> for Hash160Hashing {
	fn into(self) -> Hex {
		Hex(hex::encode(self.as_bytes()))
	}
}

impl TryFrom<Hex> for Hash160Hashing {
	type Error = StacksError;

	fn try_from(value: Hex) -> Result<Self, Self::Error> {
		Self::from_bytes(&hex::decode(value.0)?)
	}
}

/// Hash160 hasher type
pub type Hash160Hasher = Hasher<Hash160Hashing, HASH160_LENGTH>;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_sha256_hash_correctly() {
		let plaintext = "Hello world";
		let expected_hash_hex = "f5e95668dadf6fdef8521f7e1aa8a5e650c9f849";

		assert_eq!(
			hex::encode(Hash160Hasher::hash(plaintext.as_bytes())),
			expected_hash_hex
		);
	}
}

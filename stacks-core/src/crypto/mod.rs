pub use bdk::bitcoin::secp256k1;
use serde::{Deserialize, Serialize};

use crate::{StacksError, StacksResult};

/// Module for Hash160 hashing
pub mod hash160;
/// Module for sha256 hashing
pub mod sha256;
pub mod wif;

const CHECKSUM_LENGTH: usize = 4;

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct Hex(String);

/// Hashing trait
pub trait Hashing<const LENGTH: usize>: Clone + Sized {
	/// Hash the given data
	fn hash(data: &[u8]) -> Self;
	/// Get the bytes of the hash
	fn as_bytes(&self) -> &[u8];
	/// Attempt to create a hash from the given bytes
	fn from_bytes(bytes: &[u8]) -> StacksResult<Self>;
	/// Create a hash from the given bytes
	fn new(value: impl AsRef<[u8]>) -> Self {
		Self::hash(value.as_ref())
	}

	/// Create a zeroed hash
	fn zeroes() -> Self {
		Self::from_bytes(vec![0; LENGTH].as_slice()).unwrap()
	}

	/// Get the checksum of the hash
	fn checksum(&self) -> [u8; CHECKSUM_LENGTH] {
		self.as_bytes()[0..CHECKSUM_LENGTH].try_into().unwrap()
	}

	/// Attempt to create a hash from the given hex bytes
	fn from_hex(data: impl AsRef<str>) -> StacksResult<Self> {
		Self::from_bytes(&hex::decode(data.as_ref().as_bytes())?)
	}

	/// Get the hex representation of the hash
	fn to_hex(&self) -> String {
		hex::encode(self.as_bytes())
	}
}

#[derive(
	Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(try_from = "Hex")]
#[serde(into = "Hex")]
/// The hasher type
pub struct Hasher<T, const LENGTH: usize>(T)
where
	T: Hashing<LENGTH>;

impl<T, const LENGTH: usize> Hashing<LENGTH> for Hasher<T, LENGTH>
where
	T: Hashing<LENGTH>,
{
	fn hash(data: &[u8]) -> Self {
		Self(T::hash(data))
	}

	fn as_bytes(&self) -> &[u8] {
		T::as_bytes(&self.0)
	}

	fn from_bytes(bytes: &[u8]) -> StacksResult<Self> {
		Ok(Self(T::from_bytes(bytes)?))
	}
}

impl<T, const LENGTH: usize> AsRef<[u8]> for Hasher<T, LENGTH>
where
	T: Hashing<LENGTH>,
{
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl<T, const LENGTH: usize> TryFrom<&[u8]> for Hasher<T, LENGTH>
where
	T: Hashing<LENGTH>,
{
	type Error = StacksError;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		Self::from_bytes(value)
	}
}

impl<T, const LENGTH: usize> From<[u8; LENGTH]> for Hasher<T, LENGTH>
where
	T: Hashing<LENGTH>,
{
	fn from(value: [u8; LENGTH]) -> Self {
		Self::from_bytes(&value).unwrap()
	}
}

impl<T, const LENGTH: usize> Default for Hasher<T, LENGTH>
where
	T: Hashing<LENGTH>,
{
	fn default() -> Self {
		Self::zeroes()
	}
}

// From conversion is fallible for this type
#[allow(clippy::from_over_into)]
impl<T, const LENGTH: usize> Into<Hex> for Hasher<T, LENGTH>
where
	T: Hashing<LENGTH>,
{
	fn into(self) -> Hex {
		Hex(hex::encode(self.as_bytes()))
	}
}

impl<T, const LENGTH: usize> TryFrom<Hex> for Hasher<T, LENGTH>
where
	T: Hashing<LENGTH>,
{
	type Error = StacksError;

	fn try_from(value: Hex) -> Result<Self, Self::Error> {
		Self::from_bytes(&hex::decode(value.0)?)
	}
}

/// Stacks private key
pub type PrivateKey = bdk::bitcoin::secp256k1::SecretKey;

/// Stacks public key
pub type PublicKey = bdk::bitcoin::secp256k1::PublicKey;

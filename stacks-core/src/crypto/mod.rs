pub use secp256k1::*;
use serde::{Deserialize, Serialize};

use crate::{StacksError, StacksResult};

pub mod hash160;
pub mod sha256;

const CHECKSUM_LENGTH: usize = 4;

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct Hex(String);

pub trait Hashing<const LENGTH: usize>: Clone + Sized {
    fn hash(data: &[u8]) -> Self;
    fn as_bytes(&self) -> &[u8];
    fn from_bytes(bytes: &[u8]) -> StacksResult<Self>;

    fn new(value: impl AsRef<[u8]>) -> Self {
        Self::hash(value.as_ref())
    }

    fn zeroes() -> Self {
        Self::from_bytes(vec![0; LENGTH].as_slice()).unwrap()
    }

    fn checksum(&self) -> [u8; CHECKSUM_LENGTH] {
        self.as_bytes()[0..CHECKSUM_LENGTH].try_into().unwrap()
    }

    fn from_hex(data: impl AsRef<str>) -> StacksResult<Self> {
        Self::from_bytes(&hex::decode(data.as_ref().as_bytes())?)
    }

    fn to_hex(&self) -> String {
        hex::encode(self.as_bytes())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(try_from = "Hex")]
#[serde(into = "Hex")]
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

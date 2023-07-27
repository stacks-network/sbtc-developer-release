use ring::digest::Context;
use ring::digest::SHA256;
use ripemd::Digest;
use ripemd::Ripemd160;

use crate::StacksError;

const SHA256_LENGTH: usize = 32;
const CHECKSUM_LENGTH: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SHA256Hash([u8; SHA256_LENGTH]);

impl SHA256Hash {
    pub const CHECKSUM_LENGTH: usize = 4;

    pub fn new(value: impl AsRef<[u8]>) -> Self {
        let bytes = {
            let mut ctx = Context::new(&SHA256);
            ctx.update(value.as_ref());

            ctx.finish().as_ref().try_into().unwrap()
        };

        SHA256Hash(bytes)
    }

    pub fn double(value: &[u8]) -> Self {
        Self::new(Self::new(value).as_ref())
    }

    pub fn checksum(&self) -> [u8; CHECKSUM_LENGTH] {
        self.as_ref()[0..CHECKSUM_LENGTH].try_into().unwrap()
    }
}

impl AsRef<[u8]> for SHA256Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

pub const HASH160_LENGTH: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hash160(pub [u8; HASH160_LENGTH]);

impl Hash160 {
    pub fn new(value: impl AsRef<[u8]>) -> Self {
        let ripemd = Ripemd160::digest(SHA256Hash::new(value).as_ref());

        Hash160(ripemd.as_slice().try_into().unwrap())
    }

    pub fn checksum(&self) -> [u8; CHECKSUM_LENGTH] {
        self.as_ref()[0..CHECKSUM_LENGTH].try_into().unwrap()
    }
}

impl AsRef<[u8]> for Hash160 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; HASH160_LENGTH]> for Hash160 {
    fn from(value: [u8; HASH160_LENGTH]) -> Self {
        Hash160(value)
    }
}

impl TryFrom<&[u8]> for Hash160 {
    type Error = StacksError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != HASH160_LENGTH {
            return Err(StacksError::InvalidArguments(
                "Hash160 must be constructed from exactly 20 bytes",
            ));
        }

        Ok(Hash160(value.try_into().unwrap()))
    }
}

use ring::digest::Context;
use ring::digest::SHA256;
use ripemd::Digest;
use ripemd::Ripemd160;

const SHA256_LENGTH: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SHA256Hash([u8; SHA256_LENGTH]);

impl SHA256Hash {
    pub fn new(value: impl AsRef<[u8]>) -> Self {
        let bytes = {
            let mut buff = [0u8; SHA256_LENGTH];

            let mut ctx = Context::new(&SHA256);
            ctx.update(value.as_ref());

            let digest = ctx.finish();
            buff.copy_from_slice(digest.as_ref());

            buff
        };

        SHA256Hash(bytes)
    }

    pub fn double(value: &[u8]) -> Self {
        Self::new(Self::new(value).as_ref())
    }

    pub fn checksum(&self) -> [u8; 4] {
        let mut buff = [0u8; 4];

        let bytes = self.as_ref();
        buff.copy_from_slice(&bytes[0..4]);

        buff
    }
}

impl AsRef<[u8]> for SHA256Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

const HASH160_LENGTH: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hash160(pub [u8; HASH160_LENGTH]);

impl Hash160 {
    pub fn new(value: impl AsRef<[u8]>) -> Self {
        let mut buff = [0u8; HASH160_LENGTH];

        let ripemd = Ripemd160::digest(SHA256Hash::new(value).as_ref());
        buff.copy_from_slice(ripemd.as_slice());

        Hash160(buff)
    }

    pub fn checksum(&self) -> [u8; 4] {
        let mut buff = [0u8; 4];

        let bytes = self.as_ref();
        buff.copy_from_slice(&bytes[0..4]);

        buff
    }
}

impl AsRef<[u8]> for Hash160 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

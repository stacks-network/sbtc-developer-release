/*!
WIF parsing and construction of Stacks private keys.
*/

use strum::{EnumIter, FromRepr, IntoEnumIterator};

use crate::{
    crypto::{sha256::DoubleSha256Hasher, PrivateKey},
    Network, StacksError, StacksResult,
};

use super::Hashing;

/// Contains validated WIF bytes
pub struct WIF([u8; WIF_LENGTH]);

impl WIF {
    /// Constructs a WIF from a network and private key
    pub fn new(network: Network, private_key: PrivateKey) -> Self {
        let mut bytes = [0u8; WIF_LENGTH];
        bytes[0] = WIFPrefix::from(network) as u8;
        bytes[1..33].copy_from_slice(&private_key.secret_bytes());
        bytes[33] = 0x01;

        let bytes_to_hash = bytes[..34].to_vec();
        bytes[34..].copy_from_slice(&DoubleSha256Hasher::new(bytes_to_hash).as_bytes()[..4]);

        Self(bytes)
    }

    /// Attempts to parse a WIF from a byte slice
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> StacksResult<Self> {
        let bytes: [u8; WIF_LENGTH] = bytes.as_ref().try_into()?;

        let valid_network_byte = WIFPrefix::iter().any(|prefix| prefix as u8 == bytes[0]);
        let valid_private_key = PrivateKey::from_slice(&bytes[1..33]).is_ok();
        let valid_compression_byte = bytes[33] == 0x01;
        let valid_checksum = DoubleSha256Hasher::new(&bytes[..34]).as_bytes()[..4] == bytes[34..];

        if valid_network_byte && valid_private_key && valid_compression_byte && valid_checksum {
            Ok(Self(bytes))
        } else {
            Err(StacksError::InvalidData("WIF is invalid"))
        }
    }

    /// Returns the network
    pub fn network(&self) -> StacksResult<Network> {
        match WIFPrefix::from_repr(self.0[0]) {
            Some(WIFPrefix::Mainnet) => Ok(Network::Mainnet),
            Some(WIFPrefix::Testnet) => Ok(Network::Testnet),
            _ => Err(StacksError::InvalidData("Unknown network byte")),
        }
    }

    /// Returns the private key
    pub fn private_key(&self) -> StacksResult<PrivateKey> {
        Ok(PrivateKey::from_slice(&self.0[1..33])?)
    }
}

/**
WIF length consists of:

1. [WIFPrefix] byte
2. Private key bytes (32 bytes)
3. Compression byte
4. Checksum bytes ( 4 bytes)
*/
pub const WIF_LENGTH: usize = 38;

/// WIF network prefix byte
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, FromRepr)]
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

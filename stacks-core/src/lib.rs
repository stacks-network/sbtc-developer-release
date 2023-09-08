#![forbid(missing_docs)]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
/*!
# stacks-core library: a library for interacting with the Stacks protocol
*/

use std::{array::TryFromSliceError, io};

use codec::{Codec, CodecError};
use strum::{EnumIter, FromRepr};
use thiserror::Error;
use uint::Uint256;

/// Module for interacting with stacks addresses
pub mod address;
/// Module for c32 encoding and decoding
pub mod c32;
pub mod codec;
pub mod contract_name;
pub mod credentials;
/// Module for crypto functions
pub mod crypto;
/// Module for creating large integers and performing basic arithmetic
pub mod uint;
/// Module for utility functions
pub mod utils;

/// Error type for the stacks-core library
#[derive(Error, Debug)]
pub enum StacksError {
    #[error("Invalid arguments: {0}")]
    /// Invalid arguments
    InvalidArguments(&'static str),
    #[error("Could not crackford32 encode or decode: {0}")]
    /// C32 encoding or decoding error
    C32Error(#[from] c32::C32Error),
    #[error("Address version is invalid: {0}")]
    /// Invalid address version
    InvalidAddressVersion(u8),
    #[error("Could not build array from slice: {0}")]
    /// Invalid slice length
    InvalidSliceLength(#[from] TryFromSliceError),
    #[error("Could not encode or decode hex: {0}")]
    /// Hex encoding or decoding error due
    BadHex(#[from] hex::FromHexError),
    #[error("Could not create Uint from {0} bytes")]
    /// Invalid Uint bytes
    InvalidUintBytes(usize),
    #[error("Codec error: {0}")]
    /// Codec error
    CodecError(#[from] CodecError),
    #[error("Invalid data: {0}")]
    /// Invalid data
    InvalidData(&'static str),
    /// BIP32 Error
    #[error("BIP32 error: {0}")]
    BIP32(#[from] bdk::bitcoin::util::bip32::Error),
    /// BIP32 Error
    #[error("BIP39 error: {0}")]
    BIP39(#[from] bdk::keys::bip39::Error),
}

/// Result type for the stacks-core library
pub type StacksResult<T> = Result<T, StacksError>;

/// A stacks block ID
pub struct BlockId(Uint256);

impl BlockId {
    /// Creates a new StacksBlockId from a slice of bytes
    pub fn new(number: Uint256) -> StacksResult<Self> {
        Ok(Self(number))
    }
}

impl Codec for BlockId {
    fn codec_serialize<W: io::Write>(&self, dest: &mut W) -> io::Result<()> {
        self.0.codec_serialize(dest)
    }

    fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self(Uint256::codec_deserialize(data)?))
    }
}

/// Stacks network kind
#[repr(u8)]
#[derive(FromRepr, EnumIter, strum::EnumString, PartialEq, Eq, Copy, Clone, Debug)]
#[strum(ascii_case_insensitive)]
pub enum Network {
    /// Mainner
    Mainnet = 0,
    /// Testnet
    Testnet = 1,
}

/// Stacks private key
pub type PrivateKey = bdk::bitcoin::secp256k1::SecretKey;

/// Stacks public key
pub type PublicKey = bdk::bitcoin::secp256k1::PublicKey;

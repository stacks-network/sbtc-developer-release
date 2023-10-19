#![forbid(missing_docs)]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
//! # stacks-core library: a library for interacting with the Stacks protocol

use std::{array::TryFromSliceError, io};

use bdk::bitcoin::Network as BitcoinNetwork;
use codec::{Codec, CodecError};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, FromRepr};
use thiserror::Error;
use uint::Uint256;

/// Module for interacting with stacks addresses
pub mod address;
/// Module for c32 encoding and decoding
pub mod c32;
pub mod codec;
pub mod contract_name;
/// Module for crypto functions
pub mod crypto;
/// Module for creating large integers and performing basic arithmetic
pub mod uint;
/// Module for utility functions
pub mod utils;
pub mod wallet;

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
	InvalidData(String),
	/// BIP32 Error
	#[error("BIP32 error: {0}")]
	BIP32(#[from] bdk::bitcoin::util::bip32::Error),
	/// BIP32 Error
	#[error("BIP39 error: {0}")]
	BIP39(#[from] bdk::keys::bip39::Error),
	/// SECP Error
	#[error("SECP error: {0}")]
	SECP(#[from] bdk::bitcoin::secp256k1::Error),
	/// Base58 Error
	#[error("Base58 error: {0}")]
	Base58(#[from] bdk::bitcoin::util::base58::Error),
}

/// Result type for the stacks-core library
pub type StacksResult<T> = Result<T, StacksError>;

/// A stacks block ID
pub struct BlockId(Uint256);

impl BlockId {
	/// Creates a new StacksBlockId from a slice of bytes
	pub fn new(number: Uint256) -> Self {
		Self(number)
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
#[derive(
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	EnumString,
	Display,
	EnumIter,
	FromRepr,
	Serialize,
	Deserialize,
)]
#[strum(ascii_case_insensitive)]
#[strum(serialize_all = "lowercase")]
#[serde(try_from = "String", into = "String")]
pub enum Network {
	/// Mainnet
	Mainnet = 0,
	/// Testnet
	Testnet = 1,
}

impl TryFrom<String> for Network {
	type Error = strum::ParseError;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::try_from(value.as_str())
	}
}

// Other way around is fallible, so we don't implement it
#[allow(clippy::from_over_into)]
impl Into<String> for Network {
	fn into(self) -> String {
		self.to_string()
	}
}

// For some reason From impl fails to compile
#[allow(clippy::from_over_into)]
impl Into<Network> for BitcoinNetwork {
	fn into(self) -> Network {
		match self {
			BitcoinNetwork::Bitcoin => Network::Mainnet,
			_ => Network::Testnet,
		}
	}
}

// For some reason From impl fails to compile
#[allow(clippy::from_over_into)]
impl Into<BitcoinNetwork> for Network {
	fn into(self) -> BitcoinNetwork {
		match self {
			Network::Mainnet => BitcoinNetwork::Bitcoin,
			Network::Testnet => BitcoinNetwork::Testnet,
		}
	}
}

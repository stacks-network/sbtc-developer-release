#![forbid(missing_docs)]
/*!
# stacks-core library: a library for interacting with the Stacks protocol
*/

use std::array::TryFromSliceError;

use thiserror::Error;

/// Module for interacting with stacks addresses
pub mod address;
/// Module for c32 encoding and decoding
pub mod c32;
/// Module for contract name parsing
pub mod contract_name;
/// Module for crypto functions
pub mod crypto;
/// Module for creating large integers and performing basic arithmetic
pub mod uint;
/// Module for utility functions
pub mod utils;

/// Error type for the stacks-core library
#[derive(Error, Debug, Clone)]
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
}

/// Result type for the stacks-core library
pub type StacksResult<T> = Result<T, StacksError>;

use std::array::TryFromSliceError;

use thiserror::Error;

pub mod address;
pub mod c32;
pub mod crypto;
pub mod uint;
pub mod utils;

#[derive(Error, Debug, Clone)]
pub enum StacksError {
    #[error("Invalid arguments: {0}")]
    InvalidArguments(&'static str),
    #[error("Could not crackford32 encode or decode: {0}")]
    C32Error(#[from] c32::C32Error),
    #[error("Address version is invalid: {0}")]
    InvalidAddressVersion(u8),
    #[error("Could not build array from slice: {0}")]
    InvalidSliceLength(#[from] TryFromSliceError),
    #[error("Could not encode or decode hex: {0}")]
    BadHex(#[from] hex::FromHexError),
    #[error("Could not create Uint from {0} bytes")]
    InvalidUintBytes(usize),
}

pub type StacksResult<T> = Result<T, StacksError>;

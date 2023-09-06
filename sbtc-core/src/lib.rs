#![forbid(missing_docs)]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
/*!
# sbtc-core library: a library for interacting with the sBTC protocol
*/

use bdk::electrum_client::Error as ElectrumError;
use stacks_core::{contract_name::ContractNameError, StacksError};
use thiserror::Error;

/// Module for sBTC operations
pub mod operations;

/// Module for an sBTC signer
pub mod signer;

#[derive(Error, Debug)]
/// sBTC error type
pub enum SBTCError {
    #[error("Bad contract name: {0}")]
    /// Bad contract name
    BadContractName(&'static str),
    #[error("Data is malformed: {0}")]
    /// Malformed data
    MalformedData(&'static str),
    #[error("Electrum error: {0}: {1}")]
    /// Electrum error
    ElectrumError(&'static str, ElectrumError),
    #[error("BDK error: {0}: {1}")]
    /// BDK Error
    BDKError(&'static str, bdk::Error),
    #[error("Deposit amount {0} should be greater than dust amount {1}")]
    /// Insufficient amount
    AmountInsufficient(u64, u64),
    /// Contract name error
    #[error("Contract name error: {0}")]
    ContractNameError(#[from] ContractNameError),
    /// Stacks error
    #[error("Stacks error: {0}")]
    StacksError(#[from] StacksError),
    #[error("SECP error: {0}: {1}")]
    /// SECP Error
    SECPError(&'static str, bdk::bitcoin::secp256k1::Error),
    /// Not an sBTC operation
    #[error("Not an sBTC operation")]
    NotSBTCOperation,
}

/// A helper type for sBTC results
pub type SBTCResult<T> = Result<T, SBTCError>;

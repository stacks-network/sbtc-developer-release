#![deny(missing_docs)]
/*!
# sbtc-core library: a library for interacting with the sBTC protocol
*/

use bdk::electrum_client::Error as ElectrumError;
use thiserror::Error;

/// Module for sBTC operations
pub mod operations;

#[derive(Error, Debug)]
/// sBTC error type
pub enum SBTCError {
    #[error("Bad contract name: {0}")]
    /// Bad contract name
    BadContractName(&'static str),
    #[error("Data is malformed: {0}")]
    /// Malformed data
    MalformedData(&'static str),
    #[error("{0}: {1}")]
    /// Electrum error
    ElectrumError(&'static str, ElectrumError),
    #[error("{0}: {1}")]
    /// BDK Error
    BDKError(&'static str, bdk::Error),
    #[error("Deposit amount {0} should be greater than dust amount {1}")]
    /// Insufficient amount
    AmountInsufficient(u64, u64),
}

/// A helper type for sBTC results
pub type SBTCResult<T> = Result<T, SBTCError>;

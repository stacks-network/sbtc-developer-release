use bdk::electrum_client::Error as ElectrumError;
use thiserror::Error;

pub mod operations;

#[derive(Error, Debug)]
pub enum SBTCError {
    #[error("Bad contract name: {0}")]
    BadContractName(&'static str),
    #[error("Data is malformed: {0}")]
    MalformedData(&'static str),
    #[error("{0}: {1}")]
    ElectrumError(&'static str, ElectrumError),
    #[error("{0}: {1}")]
    BDKError(&'static str, bdk::Error),
    #[error("Deposit amount {0} should be greater than dust amount {1}")]
    AmountInsufficient(u64, u64),
}

pub type SBTCResult<T> = Result<T, SBTCError>;

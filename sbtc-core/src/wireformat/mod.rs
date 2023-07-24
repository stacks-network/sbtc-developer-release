use std::array::TryFromSliceError;

use thiserror::Error;

pub mod deposit;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Bad contract name: {0}")]
    BadContractName(&'static str),
    #[error("Data is malformed: {0}")]
    MalformedData(&'static str),
    #[error("Could not parse bytes")]
    Foo(#[from] TryFromSliceError),
}

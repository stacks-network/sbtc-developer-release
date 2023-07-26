use thiserror::Error;

pub mod deposit;
pub mod withdrawal_fullfillment;
pub mod withdrawal_request;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Bad contract name: {0}")]
    BadContractName(&'static str),
    #[error("Data is malformed: {0}")]
    MalformedData(&'static str),
}

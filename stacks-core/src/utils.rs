use std::{
    borrow::Borrow,
    fmt::{Display, Formatter},
    ops::Deref,
};

use once_cell::sync::Lazy;
use regex::Regex;
use thiserror::Error;

use crate::address::{AddressVersion, StacksAddress};

pub const CONTRACT_MIN_NAME_LENGTH: usize = 1;
pub const CONTRACT_MAX_NAME_LENGTH: usize = 40;

pub static CONTRACT_NAME_REGEX_STRING: Lazy<String> = Lazy::new(|| {
    format!(
        r#"([a-zA-Z](([a-zA-Z0-9]|[-_])){{{},{}}})"#,
        CONTRACT_MIN_NAME_LENGTH - 1,
        CONTRACT_MAX_NAME_LENGTH - 1
    )
});

pub static CONTRACT_NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    regex::Regex::new(format!("^{}$|^__transient$", CONTRACT_NAME_REGEX_STRING.as_str()).as_str())
        .unwrap()
});

#[derive(Error, Debug)]
pub enum ContractNameError {
    #[error(
        "Length should be between {} and {}",
        CONTRACT_MIN_NAME_LENGTH,
        CONTRACT_MAX_NAME_LENGTH
    )]
    InvalidLength,
    #[error("Format should follow the contract name specification")]
    InvalidFormat,
}

pub struct ContractName(String);

impl ContractName {
    pub fn new(contract_name: &str) -> Result<Self, ContractNameError> {
        if contract_name.len() < CONTRACT_MIN_NAME_LENGTH
            && contract_name.len() > CONTRACT_MAX_NAME_LENGTH
        {
            Err(ContractNameError::InvalidLength)
        } else if CONTRACT_NAME_REGEX.is_match(contract_name) {
            Ok(Self(contract_name.to_string()))
        } else {
            Err(ContractNameError::InvalidFormat)
        }
    }
}

impl TryFrom<&str> for ContractName {
    type Error = ContractNameError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ContractName::new(value)
    }
}

impl AsRef<str> for ContractName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Deref for ContractName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Borrow<str> for ContractName {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

#[allow(clippy::from_over_into)]
impl Into<String> for ContractName {
    fn into(self) -> String {
        self.0
    }
}

impl Display for ContractName {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub struct StandardPrincipalData(AddressVersion, StacksAddress);

impl StandardPrincipalData {
    pub fn new(version: AddressVersion, address: StacksAddress) -> Self {
        Self(version, address)
    }
}

pub enum PrincipalData {
    Standard(StandardPrincipalData),
    Contract(StandardPrincipalData, ContractName),
}

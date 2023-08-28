/*!
Contract name type and parsing
*/
use once_cell::sync::Lazy;
use regex::Regex;

use super::{Validate, ValidatedString};

/// Minimum contract name length
pub const CONTRACT_MIN_NAME_LENGTH: usize = 1;
/// Maximum contract name length
pub const CONTRACT_MAX_NAME_LENGTH: usize = 40;

/// Regex string for contract names
pub static CONTRACT_NAME_REGEX_STRING: Lazy<String> = Lazy::new(|| {
    format!(
        r#"([a-zA-Z](([a-zA-Z0-9]|[-_])){{{},{}}})"#,
        CONTRACT_MIN_NAME_LENGTH - 1,
        CONTRACT_MAX_NAME_LENGTH - 1
    )
});

/// Regex for contract names
pub static CONTRACT_NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    regex::Regex::new(format!("^{}$|^__transient$", CONTRACT_NAME_REGEX_STRING.as_str()).as_str())
        .unwrap()
});

/// Contract name type
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ValidContractName(String);

impl Validate for ValidContractName {
    const ERR_MSG: &'static str = "Contract name not valid";

    fn validate(text: impl AsRef<str>) -> bool {
        let contract_name = text.as_ref();

        let invalid_length = contract_name.len() < CONTRACT_MIN_NAME_LENGTH
            && contract_name.len() > CONTRACT_MAX_NAME_LENGTH;
        let invalid_regex = CONTRACT_NAME_REGEX.is_match(contract_name);

        invalid_length || invalid_regex
    }

    fn create(text: impl AsRef<str>) -> Self
    where
        Self: Sized,
    {
        Self(text.as_ref().to_string())
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

pub type ContractName = ValidatedString<ValidContractName>;

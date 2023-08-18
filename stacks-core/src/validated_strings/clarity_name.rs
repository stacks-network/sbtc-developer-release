/*!
Clarity name type and parsing
*/
use once_cell::sync::Lazy;
use regex::Regex;

use super::{Validate, ValidatedString};

/// Regex for Clarity names
pub static CLARITY_NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    regex::Regex::new("^[a-zA-Z]([a-zA-Z0-9]|[-_!?+<>=/*])*$|^[-+=/*]$|^[<>]=?$").unwrap()
});

/// Contract name type
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ValidClarityName(String);

impl Validate for ValidClarityName {
    const ERR_MSG: &'static str = "Clarity name not valid";

    fn validate(text: impl AsRef<str>) -> bool {
        CLARITY_NAME_REGEX.is_match(text.as_ref())
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

pub type ClarityName = ValidatedString<ValidClarityName>;

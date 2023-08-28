/*!
Clarity name type and parsing
*/
use super::{Validate, ValidatedString};

/// Validated Stacks string type
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ValidStacksString(String);

impl Validate for ValidStacksString {
    const ERR_MSG: &'static str = "Clarity name not valid";

    fn validate(text: impl AsRef<str>) -> bool {
        let text = text.as_ref();

        text.is_ascii() && text.len() <= u32::MAX as usize
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

pub type StacksString = ValidatedString<ValidStacksString>;

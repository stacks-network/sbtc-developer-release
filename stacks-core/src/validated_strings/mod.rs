use std::{
    borrow::Borrow,
    fmt,
    io::{self, Read},
    ops::Deref,
};

use crate::{codec::Codec, StacksError, StacksResult};

pub mod clarity_name;
pub mod contract_name;
pub mod stacks_string;

/// Specifies how to validate a string for a type
pub trait Validate: fmt::Debug + Clone + PartialEq + Eq {
    const ERR_MSG: &'static str;

    /// Validate string
    fn validate(text: impl AsRef<str>) -> bool;

    /// Create type from valid string
    fn create(text: impl AsRef<str>) -> Self
    where
        Self: Sized;

    /// Return a reference to the valid string
    fn as_str(&self) -> &str;

    fn new(text: impl AsRef<str>) -> StacksResult<Self>
    where
        Self: Sized,
    {
        if Self::validate(text) {
            Ok(Self::create(text))
        } else {
            Err(crate::StacksError::InvalidArguments(Self::ERR_MSG))
        }
    }
}

/// Validated string of some type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedString<T>(T);

impl<T> Validate for ValidatedString<T>
where
    T: Validate + fmt::Debug + Clone + PartialEq + Eq,
{
    const ERR_MSG: &'static str = T::ERR_MSG;

    fn validate(text: impl AsRef<str>) -> bool {
        T::validate(text)
    }

    fn create(text: impl AsRef<str>) -> Self
    where
        Self: Sized,
    {
        Self(T::create(text))
    }

    fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<T> Codec for ValidatedString<T>
where
    T: Validate,
{
    fn codec_serialize<W: io::Write>(&self, dest: &mut W) -> io::Result<()> {
        dest.write_all(&[self.as_str().len() as u8])?;
        dest.write_all(self.as_str().as_bytes())
    }

    fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
    where
        Self: Sized,
    {
        let mut length_buffer = [0u8; 1];
        data.read_exact(&mut length_buffer)?;
        let contract_name_length = length_buffer[0] as usize;

        let mut name_buffer = Vec::with_capacity(contract_name_length);
        data.take(contract_name_length as u64)
            .read_to_end(&mut name_buffer)?;

        let contract_name_string = String::from_utf8(name_buffer)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        Self::new(&contract_name_string)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }
}

impl<T> TryFrom<&str> for ValidatedString<T>
where
    T: Validate,
{
    type Error = StacksError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<T> AsRef<str> for ValidatedString<T>
where
    T: Validate,
{
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl<T> Deref for ValidatedString<T>
where
    T: Validate,
{
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl<T> Borrow<str> for ValidatedString<T>
where
    T: Validate,
{
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

/*!
Module for serializing and deserializing Stacks data types
*/
use std::io;

use thiserror::Error;

use crate::StacksResult;

#[derive(Error, Debug)]
/// Codec error
pub enum CodecError {
    #[error("Could not serialize or deserialize: {0}")]
    /// Io error
    IoError(#[from] io::Error),
}

/// Codec result
pub type CodecResult<T> = Result<T, CodecError>;

/// Serializing and deserializing Stacks data types
pub trait Codec {
    /// Serialize to a writer
    fn codec_serialize<W: io::Write>(&self, dest: &mut W) -> io::Result<()>;
    /// Deserialize from a reader
    fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
    where
        Self: Sized;

    /// Serialize to a writer and return a StacksResult
    fn serialize<W: io::Write>(&self, dest: &mut W) -> StacksResult<()> {
        self.codec_serialize(dest)
            .map_err(|err| CodecError::IoError(err).into())
    }

    /// Deserialize from a reader and return a StacksResult
    fn deserialize<R: io::Read>(data: &mut R) -> StacksResult<Self>
    where
        Self: Sized,
    {
        Self::codec_deserialize(data).map_err(|err| CodecError::IoError(err).into())
    }

    /// Serialize to a vector
    fn serialize_to_vec(&self) -> Vec<u8> {
        let mut buffer = vec![];

        self.serialize(&mut buffer).unwrap();

        buffer
    }
}

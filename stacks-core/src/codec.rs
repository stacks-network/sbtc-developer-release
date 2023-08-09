/**
Module for contract name parsing
*/
use std::io;

use thiserror::Error;

use crate::StacksResult;

#[derive(Error, Debug)]
pub enum CodecError {
    #[error("Could not serialize or deserialize: {0}")]
    IoError(#[from] io::Error),
}

pub type CodecResult<T> = Result<T, CodecError>;

pub trait Codec {
    fn codec_serialize<W: io::Write>(&self, dest: &mut W) -> io::Result<()>;
    fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
    where
        Self: Sized;

    fn serialize<W: io::Write>(&self, dest: &mut W) -> StacksResult<()> {
        self.codec_serialize(dest)
            .map_err(|err| CodecError::IoError(err).into())
    }

    fn deserialize<R: io::Read>(data: &mut R) -> StacksResult<Self>
    where
        Self: Sized,
    {
        Self::codec_deserialize(data).map_err(|err| CodecError::IoError(err).into())
    }

    fn serialize_to_vec(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(21);

        self.serialize(&mut buffer).unwrap();

        buffer
    }
}

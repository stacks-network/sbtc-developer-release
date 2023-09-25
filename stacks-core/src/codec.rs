//! Module for serializing and deserializing Stacks data types
use std::io;

use bdk::bitcoin::{
	secp256k1::ecdsa::{RecoverableSignature, RecoveryId},
	Amount,
};
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
		Self::codec_deserialize(data)
			.map_err(|err| CodecError::IoError(err).into())
	}

	/// Serialize to a vector
	fn serialize_to_vec(&self) -> Vec<u8> {
		let mut buffer = vec![];

		self.serialize(&mut buffer).unwrap();

		buffer
	}
}

impl Codec for Amount {
	fn codec_serialize<W: io::Write>(&self, dest: &mut W) -> io::Result<()> {
		dest.write_all(&self.to_sat().to_be_bytes())
	}

	fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
	where
		Self: Sized,
	{
		let mut buffer = [0; 8];
		data.read_exact(&mut buffer)?;

		Ok(Self::from_sat(u64::from_be_bytes(buffer)))
	}
}

impl Codec for RecoverableSignature {
	fn codec_serialize<W: io::Write>(&self, dest: &mut W) -> io::Result<()> {
		let (id, signature) = self.serialize_compact();

		let id: u8 = id.to_i32().try_into().unwrap();

		dest.write_all(&[id])?;
		dest.write_all(&signature)
	}

	fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
	where
		Self: Sized,
	{
		let mut id_buffer = [0; 1];
		data.read_exact(&mut id_buffer)?;

		let id = RecoveryId::from_i32(id_buffer[0] as i32)
			.map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

		let mut signature_buffer = [0; 64];
		data.read_exact(&mut signature_buffer)?;

		Self::from_compact(&signature_buffer, id)
			.map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
	}
}

//! Module for serializing and deserializing Stacks data types
use std::io;

use bdk::bitcoin::{
	secp256k1::ecdsa::{RecoverableSignature, RecoveryId},
	Amount, Script,
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

impl Codec for u64 {
	fn codec_serialize<W: io::Write>(&self, dest: &mut W) -> io::Result<()> {
		dest.write_all(&self.to_be_bytes())
	}

	fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
	where
		Self: Sized,
	{
		let mut bytes = [0; 8];
		data.read_exact(&mut bytes)?;

		Ok(Self::from_be_bytes(bytes))
	}
}

impl Codec for Script {
	fn codec_serialize<W: io::Write>(&self, dest: &mut W) -> io::Result<()> {
		dest.write_all(self.as_bytes())
	}

	fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
	where
		Self: Sized,
	{
		let mut buffer = vec![];
		data.read_to_end(&mut buffer)?;

		Ok(Self::from(buffer))
	}
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use bdk::bitcoin::{
		secp256k1::{Message, Secp256k1, SecretKey},
		Amount,
	};

	use crate::StacksError;

	use super::*;

	#[test]
	fn should_serialize_amount() -> anyhow::Result<()> {
		let amount = Amount::from_sat(10_000);
		let mut serialized_amount = vec![];

		amount.serialize(&mut serialized_amount)?;

		assert_eq!(hex::encode(serialized_amount), "0000000000002710");

		Ok(())
	}

	#[test]
	fn should_deserialize_amount() -> anyhow::Result<()> {
		let mut serialized_amount =
			Cursor::new(hex::decode("0000000000002710")?);

		let deserialized_amount = Amount::deserialize(&mut serialized_amount)?;

		assert_eq!(deserialized_amount.to_sat(), 10_000);

		Ok(())
	}

	#[test]
	fn should_serialize_recoverable_signature() -> anyhow::Result<()> {
		let signature = get_recoverable_signature()?;
		let mut serialized_signature = vec![];

		signature.serialize(&mut serialized_signature)?;

		assert_eq!(hex::encode(serialized_signature), "0119874ebfb457c08cedb5ebf01fe13bf4b6ac216b6f4044763ad95a69022bf1ba3cdba26d7ebb695a7144c8de4ba672dddfc602ffa9e62a745d8f7e4206ae6a93");

		Ok(())
	}

	#[test]
	fn should_deserialize_recoverable_signature() -> anyhow::Result<()> {
		let mut serialized_signature = Cursor::new(
			hex::decode("0119874ebfb457c08cedb5ebf01fe13bf4b6ac216b6f4044763ad95a69022bf1ba3cdba26d7ebb695a7144c8de4ba672dddfc602ffa9e62a745d8f7e4206ae6a93")?
		);

		let signature =
			RecoverableSignature::deserialize(&mut serialized_signature)?;

		let expected_signature = get_recoverable_signature()?;

		assert_eq!(signature, expected_signature);

		Ok(())
	}

	#[test]
	fn should_fail_deserialize_recoverable_signature_with_invalid_id(
	) -> anyhow::Result<()> {
		let mut invalid_serialized_signature = Cursor::new(vec![4]);

		let result = RecoverableSignature::deserialize(
			&mut invalid_serialized_signature,
		);

		match result {
			Err(StacksError::CodecError(_)) => Ok(()),
			Err(e) => {
				panic!("Expected invalid recovery ID error, but got {:?}", e)
			}
			Ok(_) => panic!("Expected invalid recovery ID error, but got Ok"),
		}
	}

	#[test]
	fn should_fail_deserialize_recoverable_signature_with_invalid_signature(
	) -> anyhow::Result<()> {
		let mut invalid_serialized_signature = vec![0; 65];

		invalid_serialized_signature[0] = 1;

		for i in 1..65 {
			invalid_serialized_signature[i] = 255;
		}

		let result = RecoverableSignature::deserialize(&mut Cursor::new(
			invalid_serialized_signature,
		));

		match result {
			Err(StacksError::CodecError(_)) => Ok(()),
			Err(e) => panic!("Expected invalid signature error, got {:?}", e),
			Ok(_) => panic!("Expected invalid signature error, but got Ok"),
		}
	}

	#[test]
	fn should_serialize_u64() -> anyhow::Result<()> {
		let mut serialized_u64 = vec![];

		10_000u64.serialize(&mut serialized_u64)?;

		assert_eq!(hex::encode(serialized_u64), "0000000000002710");

		Ok(())
	}

	#[test]
	fn should_deserialize_u64() -> anyhow::Result<()> {
		let mut serialized_amount =
			Cursor::new(hex::decode("0000000000002710")?);

		let deserialized_amount = u64::deserialize(&mut serialized_amount)?;

		assert_eq!(deserialized_amount, 10_000u64);

		Ok(())
	}

	fn get_recoverable_signature() -> anyhow::Result<RecoverableSignature> {
		let secp = Secp256k1::new();

		let secret_key_bytes = hex::decode(
			"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
		)?;

		let secret_key = SecretKey::from_slice(&secret_key_bytes)?;

		let message = Message::from_slice(&mut hex::decode(
			"1bf9ad7ce49adf6cbc707a689b6e17653151e95c1cd8a53f9fce54d3d51a2a24",
		)?)?;

		let recoverable_signature =
			secp.sign_ecdsa_recoverable(&message, &secret_key);

		Ok(recoverable_signature)
	}
}

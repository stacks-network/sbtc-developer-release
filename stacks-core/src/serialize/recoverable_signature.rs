//! A module providing serialization for [BitcoinRecoverableSignature]

use std::io;

use bdk::bitcoin::secp256k1::ecdsa::{
	RecoverableSignature as BitcoinRecoverableSignature, RecoveryId,
};

use super::{DeserializeBytes, SerializeBytes};

impl SerializeBytes for BitcoinRecoverableSignature {
	fn write_buffer<WritableBuffer: std::io::Write>(
		&self,
		buffer: &mut WritableBuffer,
	) -> std::io::Result<()> {
		let (id, signature) = self.serialize_compact();

		let id: u8 = id.to_i32().try_into().unwrap();

		buffer.write_all(&[id])?;
		buffer.write_all(&signature)
	}
}

impl DeserializeBytes for BitcoinRecoverableSignature {
	fn read_buffer<ReadableBuffer: std::io::Read>(
		buffer: &mut ReadableBuffer,
	) -> std::io::Result<Self>
	where
		Self: Sized,
	{
		let mut id = [0; 1];
		buffer.read_exact(&mut id)?;

		let id = RecoveryId::from_i32(id[0] as i32)
			.map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

		let mut signature_buffer = [0; 64];
		buffer.read_exact(&mut signature_buffer)?;

		Self::from_compact(&signature_buffer, id)
			.map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::io::Cursor;

	use bdk::bitcoin::secp256k1::{Message, Secp256k1, SecretKey};

	use crate::StacksError;

	#[test]
	fn should_serialize_recoverable_signature() {
		let signature = get_recoverable_signature().unwrap();
		let mut serialized_signature = vec![];

		signature.serialize(&mut serialized_signature).unwrap();

		let expected_serialized_signature = hex::decode("0119874ebfb457c08cedb5ebf01fe13bf4b6ac216b6f4044763ad95a69022bf1ba3cdba26d7ebb695a7144c8de4ba672dddfc602ffa9e62a745d8f7e4206ae6a93").unwrap();

		assert_eq!(serialized_signature, expected_serialized_signature);
	}

	#[test]
	fn should_deserialize_recoverable_signature() {
		let mut serialized_signature = Cursor::new(
			hex::decode("0119874ebfb457c08cedb5ebf01fe13bf4b6ac216b6f4044763ad95a69022bf1ba3cdba26d7ebb695a7144c8de4ba672dddfc602ffa9e62a745d8f7e4206ae6a93").unwrap()
		);

		let signature =
			BitcoinRecoverableSignature::deserialize(&mut serialized_signature)
				.unwrap();

		let expected_signature = get_recoverable_signature().unwrap();

		assert_eq!(signature, expected_signature);
	}

	#[test]
	fn should_fail_deserialize_recoverable_signature_with_recovery_id_bytes_out_of_bounds(
	) {
		let mut invalid_serialized_signature = Cursor::new(vec![4]);

		let result = BitcoinRecoverableSignature::deserialize(
			&mut invalid_serialized_signature,
		);

		match result {
			Err(StacksError::SerializationError(_)) => {}
			Err(e) => {
				panic!("Expected SerializationError, but got {:?}", e)
			}
			Ok(_) => panic!("Expected invalid recovery ID error, but got Ok"),
		}
	}

	#[test]
	fn should_fail_deserialize_recoverable_signature_with_signature_bytes_non_ecdsa(
	) {
		let mut invalid_serialized_signature = vec![0; 65];

		invalid_serialized_signature[0] = 1;

		for i in 1..65 {
			invalid_serialized_signature[i] = 255;
		}

		let result = BitcoinRecoverableSignature::deserialize(
			&mut Cursor::new(invalid_serialized_signature),
		);

		match result {
			Err(StacksError::SerializationError(_)) => {}
			Err(e) => panic!("Expected SerializationError, got {:?}", e),
			Ok(_) => panic!("Expected invalid signature error, but got Ok"),
		}
	}

	fn get_recoverable_signature() -> anyhow::Result<BitcoinRecoverableSignature>
	{
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

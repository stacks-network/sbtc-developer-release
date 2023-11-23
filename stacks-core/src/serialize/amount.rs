//! A module providing serialization for [BitcoinAmount].

use std::io;

use bdk::bitcoin::Amount as BitcoinAmount;

use super::{DeserializeBytes, SerializeBytes};

impl SerializeBytes for BitcoinAmount {
	fn write_buffer<WritableBuffer: io::Write>(
		&self,
		buffer: &mut WritableBuffer,
	) -> io::Result<()> {
		buffer.write_all(&self.to_sat().to_be_bytes())
	}
}

impl DeserializeBytes for BitcoinAmount {
	fn read_buffer<ReadableBuffer: io::Read>(
		buffer: &mut ReadableBuffer,
	) -> io::Result<Self>
	where
		Self: Sized,
	{
		let mut amount_bytes = [0; 8];
		buffer.read_exact(&mut amount_bytes)?;

		Ok(Self::from_sat(u64::from_be_bytes(amount_bytes)))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::io::Cursor;

	#[test]
	fn should_serialize_amount() {
		let amount = BitcoinAmount::from_sat(10_000);
		let mut serialized_amount = vec![];

		amount.serialize(&mut serialized_amount).unwrap();

		let expected_serialized_amount =
			hex::decode("0000000000002710").unwrap();

		assert_eq!(serialized_amount, expected_serialized_amount);
	}

	#[test]
	fn should_deserialize_amount() {
		let mut serialized_amount =
			Cursor::new(hex::decode("0000000000002710").unwrap());

		let deserialized_amount =
			BitcoinAmount::deserialize(&mut serialized_amount).unwrap();

		let expected_deserialized_amount = BitcoinAmount::from_sat(10_000);

		assert_eq!(deserialized_amount, expected_deserialized_amount);
	}
}

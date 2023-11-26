//! A module for serialization and deserialization of primitives used by sBTC.

use std::io;
use thiserror::Error;

use crate::StacksResult;

pub mod amount;
pub mod recoverable_signature;

/// Errors resulting from serialization.
#[derive(Error, Debug)]
pub enum SerializationError {
	/// Error resulting from failing to read or write data from a buffer.
	#[error("Could not serialize or deserialize: {0}")]
	IoError(#[from] io::Error),
}

/// A trait for serializing data into a writable buffer.
///
/// [SerializeBytes] provides a mechanism to convert an instance of a type
/// into a series of bytes that can be written to an output stream or buffer.
/// This is useful when precise control of the byte structure is required during
/// serialization.
pub trait SerializeBytes {
	/// Serializes the invoking object into the provided writable buffer.
	fn serialize<WritableBuffer: io::Write>(
		&self,
		dest: &mut WritableBuffer,
	) -> StacksResult<()> {
		self.write_buffer(dest)
			.map_err(|err| SerializationError::IoError(err).into())
	}

	/// Writes data to a buffer, used for deserialization.
	///
	/// # ⚠️ Do not use directly
	///
	/// This function is not supposed to be called directly. Doing so can cause
	/// unexpected effects like improper error handling.
	fn write_buffer<WritableBuffer: io::Write>(
		&self,
		buffer: &mut WritableBuffer,
	) -> io::Result<()>;
}

/// A trait for deserializing data from a readable buffer.
///
/// [DeserializeBytes] provides a mechanism to convert a series of bytes that can be
/// read from an output stream or buffer into an instance of a type. This is useful
/// when precise control of the byte structure is required during serialization.
pub trait DeserializeBytes {
	/// Deserializes an object from the provided readable buffer.
	fn deserialize<ReadableBuffer: io::Read>(
		src: &mut ReadableBuffer,
	) -> StacksResult<Self>
	where
		Self: Sized,
	{
		Self::read_buffer(src)
			.map_err(|err| SerializationError::IoError(err).into())
	}

	/// Reads data from a buffer, used for deserialization.
	///
	/// # ⚠️ Do not use directly
	///
	/// This function is not supposed to be called directly. Doing so can cause
	/// unexpected effects like improper error handling.
	fn read_buffer<ReadableBuffer: io::Read>(
		buffer: &mut ReadableBuffer,
	) -> io::Result<Self>
	where
		Self: Sized;
}

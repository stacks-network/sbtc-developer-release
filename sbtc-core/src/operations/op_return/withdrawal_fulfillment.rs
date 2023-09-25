//! Tools for the construction and parsing of the sBTC OP_RETURN withdrawal
//! fulfillment transactions.
//!
//! Withdrawal fulfillment is a Bitcoin transaction with the output structure as
//! below:
//!
//! 1. data output
//! 2. Bitcoin address to send the BTC to
//!
//! The data output should contain data in the following byte format:
//!
//! ```text
//! 0     2  3                                                                    80
//! |-----|--|---------------------------------------------------------------------|
//! magic op                      withdrawal fulfillment data
//! ```
//!
//! Where withdrawal fulfillment data should be in the following format:
//!
//! ```text
//! 3                             35                                              80
//! |------------------------------|-----------------------------------------------|
//! chain tip                             extra bytes

use std::io;

use stacks_core::{codec::Codec, BlockId};

/// The parsed data output from a withdrawal fulfillment transaction
pub struct ParsedWithdrawalFulfillmentData {
	/// The chain tip block ID
	pub chain_tip: BlockId,
}

impl Codec for ParsedWithdrawalFulfillmentData {
	fn codec_serialize<W: io::Write>(&self, dest: &mut W) -> io::Result<()> {
		self.chain_tip.codec_serialize(dest)
	}

	fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {
			chain_tip: BlockId::codec_deserialize(data)?,
		})
	}
}

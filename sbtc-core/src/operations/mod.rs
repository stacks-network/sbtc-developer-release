use std::io;

use bdk::bitcoin::Network;
use stacks_core::codec::Codec;
use strum::FromRepr;

pub mod commit_reveal;
pub mod op_return;
pub mod utils;

/// Opcodes of sBTC transactions
#[derive(FromRepr, Debug, Clone, Copy)]
#[repr(u8)]
pub enum Opcode {
	/// Deposit
	Deposit = b'<',
	/// Withdrawal request
	WithdrawalRequest = b'>',
	/// Withdrawal fulfillment
	WithdrawalFulfillment = b'!',
	/// Wallet handoff
	WalletHandoff = b'H',
}

impl Codec for Opcode {
	fn codec_serialize<W: io::Write>(&self, dest: &mut W) -> io::Result<()> {
		dest.write_all(&[*self as u8])
	}

	fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
	where
		Self: Sized,
	{
		let mut buffer = [0; 1];
		data.read_exact(&mut buffer)?;

		Self::from_repr(buffer[0])
			.ok_or(io::Error::new(io::ErrorKind::InvalidData, "Invalid opcode"))
	}
}

/// Returns the magic bytes for the provided network
pub(crate) fn magic_bytes(network: Network) -> [u8; 2] {
	match network {
		Network::Bitcoin => [b'X', b'2'],
		Network::Testnet => [b'T', b'2'],
		_ => [b'i', b'd'],
	}
}

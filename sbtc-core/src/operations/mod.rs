use bdk::bitcoin::Network;
use strum::FromRepr;

/// Primitives for commit reveal transactions
pub mod commit_reveal;
/// Primitives for op return transactions
pub mod op_return;

/// Opcodes of sBTC transactions
#[derive(FromRepr, Debug)]
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

/// Returns the magic bytes for the provided network
pub(crate) fn magic_bytes(network: Network) -> [u8; 2] {
    match network {
        Network::Bitcoin => [b'X', b'2'],
        Network::Testnet => [b'T', b'2'],
        _ => [b'i', b'd'],
    }
}

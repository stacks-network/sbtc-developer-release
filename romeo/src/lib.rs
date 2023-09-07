//! # Romeo: sBTC Developers best friend
//!
//! This binary emulates a working sBTC system.
//! When pointed at a Bitcoin and a Stacks node,
//! this system will monitor Bitcoin for sBTC operations
//! and respond the same way the final sBTC system is intended to.
#![forbid(missing_docs)]

pub mod bitcoin_client;
pub mod config;
pub mod event;
pub mod proof_data;
pub mod stacks_client;
pub mod state;
pub mod system;
pub mod task;

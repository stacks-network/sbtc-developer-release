//! # Romeo: sBTC Developers best friend
//!
//! This binary emulates a working sBTC system.
//! When pointed at a Bitcoin and a Stacks node,
//! this system will monitor Bitcoin for sBTC operations
//! and respond the same way the final sBTC system is intended to.
#![forbid(missing_docs)]

/// Configuration
pub mod config;

/// Event
pub mod event;

/// State
pub mod state;

/// System
pub mod system;

/// Task
pub mod task;

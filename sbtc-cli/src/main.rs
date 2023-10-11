#![forbid(missing_docs)]

//! sBTC CLI is a tool that allows you to generate and broadcast sBTC
//! transactions.
//!
//! It also allows you to generate credentials needed to generate transactions
//! and interact with the Bitcoin and Stacks networks.

use clap::{Parser, Subcommand};
use sbtc_cli::commands::{
	broadcast::{broadcast_tx, BroadcastArgs},
	deposit::{build_deposit_tx, DepositArgs},
	generate::{generate, GenerateArgs},
	withdraw::{build_withdrawal_tx, WithdrawalArgs},
};

#[derive(Parser)]
struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
	Deposit(DepositArgs),
	Withdraw(WithdrawalArgs),
	Broadcast(BroadcastArgs),
	GenerateFrom(GenerateArgs),
}

fn main() -> Result<(), anyhow::Error> {
	let args = Cli::parse();

	match args.command {
		Command::Deposit(deposit_args) => build_deposit_tx(&deposit_args),
		Command::Withdraw(withdrawal_args) => {
			build_withdrawal_tx(&withdrawal_args)
		}
		Command::Broadcast(broadcast_args) => broadcast_tx(&broadcast_args),
		Command::GenerateFrom(generate_args) => generate(&generate_args),
	}
}

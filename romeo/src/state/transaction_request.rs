//! Transaction request state and its utils.
use std::fmt::Display;

use tracing::debug;

use super::TransactionStatus;

/// A transaction request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) enum TransactionRequest<T> {
	/// Scheduled to be created at a given stacks block height.
	Scheduled {
		/// The stacks block height at which the transaction should be created.
		block_height: u32,
	},
	/// Created and passed on to a task
	Created,
	/// Acknowledged by a task with the status update
	Acknowledged(Acknowledged<T>),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Acknowledged<T> {
	/// The transaction ID
	pub txid: T,
	/// The status of the transaction
	pub status: TransactionStatus,
	/// Whether the task has a pending request
	pub has_pending_task: bool,
}

impl<Txid> TransactionRequest<Txid> {
	pub(super) fn filtered_acknowledged_ref_mut(
		&mut self,
		txid: Txid,
		strict: bool,
		status: &TransactionStatus,
	) -> Option<Result<&mut Acknowledged<Txid>, EarlyExit>>
	where
		Txid: PartialEq + Display,
	{
		let TransactionRequest::Acknowledged(ack) = self else {
			debug!("Skipping tx not acknowledged yet");
			return None;
		};

		if txid != ack.txid {
			return Some(Err(EarlyExit::NotSought));
		}

		if !ack.has_pending_task {
			if strict {
				panic!(
			            "Got an {:?} status update for a transaction that doesn't have a pending task: {}", status, txid
			        );
			} else {
				debug!(
			            "Ignoring {:?} status update for a transaction that doesn't have a pending task: {}", status, txid
			        );
				return Some(Err(EarlyExit::NotPending));
			}
		}

		Some(Ok(ack))
	}
}

#[derive(Debug)]
pub(super) enum EarlyExit {
	NotSought,
	NotPending,
}

#[cfg(test)]
mod tests {
	use assert_matches::assert_matches;

	use super::*;

	#[test]
	fn request_not_ack() {
		let mut t_r = TransactionRequest::Created;
		assert_matches!(
			t_r.filtered_acknowledged_ref_mut(
				"",
				false,
				&TransactionStatus::Confirmed,
			),
			None
		);

		let mut t_r = TransactionRequest::Scheduled { block_height: 0 };
		assert_matches!(
			t_r.filtered_acknowledged_ref_mut(
				"",
				false,
				&TransactionStatus::Confirmed,
			),
			None
		);
	}

	#[test]
	fn request_not_sought() {
		let mut t_r = TransactionRequest::Acknowledged(Acknowledged {
			txid: "someTxid",
			status: TransactionStatus::Broadcasted,
			has_pending_task: true,
		});
		assert_matches!(
			t_r.filtered_acknowledged_ref_mut(
				"someOtherTxid",
				false,
				&TransactionStatus::Broadcasted,
			)
			.unwrap(),
			Err(EarlyExit::NotSought)
		);
	}

	#[test]
	fn request_not_pending() {
		let mut t_r = TransactionRequest::Acknowledged(Acknowledged {
			txid: "someTxid",
			status: TransactionStatus::Broadcasted,
			has_pending_task: false,
		});
		assert_matches!(
			t_r.filtered_acknowledged_ref_mut(
				"someTxid",
				false,
				&TransactionStatus::Broadcasted,
			)
			.unwrap(),
			Err(EarlyExit::NotPending)
		);
	}

	#[test]
	fn filter_request_ok_not_mutated() {
		let txid = "someTxid";
		let status = TransactionStatus::Broadcasted;
		let has_pending_task = true;
		let mut t_r = TransactionRequest::Acknowledged(Acknowledged {
			txid,
			status: status.clone(),
			has_pending_task,
		});
		assert_matches!(
			t_r.filtered_acknowledged_ref_mut(
				txid,
				false,
				&status,
			)
			.unwrap(),
			Ok(Acknowledged {
				txid:a,
				status:b,
				has_pending_task:c
			})=>{
				assert_eq!(&txid,a);
				assert_eq!(&status,b);
				assert_eq!(&has_pending_task,c);
			}
		);
	}
}

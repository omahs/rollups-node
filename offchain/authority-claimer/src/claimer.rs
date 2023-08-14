// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use async_trait::async_trait;
use rollups_events::RollupsClaim;
use snafu::ResultExt;
use std::fmt::Debug;
use tracing::{info, trace};

use crate::{checker::DuplicateChecker, sender::TransactionSender};

/// The `Claimer` sends claims to the blockchain. It checks
/// whether the claim is duplicated before sending.
#[async_trait]
pub trait Claimer: Sized + Debug {
    type Error: snafu::Error + 'static;

    async fn send_rollups_claim(
        self,
        rollups_claim: RollupsClaim,
    ) -> Result<Self, Self::Error>;
}

#[derive(Debug, snafu::Snafu)]
pub enum ClaimerError<D: DuplicateChecker, T: TransactionSender> {
    #[snafu(display("duplicated claim error"))]
    DuplicatedClaimError { source: D::Error },

    #[snafu(display("transaction sender error"))]
    TransactionSenderError { source: T::Error },
}

/// The `AbstractClaimer` must be injected with a
/// `DuplicateChecker` and a `TransactionSender`.
#[derive(Debug)]
pub struct AbstractClaimer<D: DuplicateChecker, T: TransactionSender> {
    duplicate_checker: D,
    transaction_sender: T,
}

impl<D: DuplicateChecker, T: TransactionSender> AbstractClaimer<D, T> {
    pub fn new(duplicate_checker: D, transaction_sender: T) -> Self {
        Self {
            duplicate_checker,
            transaction_sender,
        }
    }
}

#[async_trait]
impl<D, T> Claimer for AbstractClaimer<D, T>
where
    D: DuplicateChecker + Send + Sync + 'static,
    T: TransactionSender + Send + 'static,
{
    type Error = ClaimerError<D, T>;

    async fn send_rollups_claim(
        mut self,
        rollups_claim: RollupsClaim,
    ) -> Result<Self, Self::Error> {
        let is_duplicated_rollups_claim = self
            .duplicate_checker
            .is_duplicated_rollups_claim(&rollups_claim)
            .await
            .context(DuplicatedClaimSnafu)?;
        if is_duplicated_rollups_claim {
            trace!("It was a duplicated claim");
            return Ok(self);
        }

        info!("Sending a new rollups claim");
        self.transaction_sender = self
            .transaction_sender
            .send_rollups_claim_transaction(rollups_claim)
            .await
            .context(TransactionSenderSnafu)?;

        Ok(self)
    }
}

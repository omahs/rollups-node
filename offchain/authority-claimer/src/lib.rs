// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pub mod checker;
pub mod claimer;
pub mod config;
pub mod listener;
pub mod metrics;
pub mod sender;

#[cfg(test)]
mod mock;

use config::Config;
use rollups_events::DAppMetadata;
use snafu::Error;
use tracing::trace;

use crate::{
    checker::DefaultDuplicateChecker,
    claimer::AbstractClaimer,
    listener::{BrokerListener, DefaultBrokerListener},
    metrics::AuthorityClaimerMetrics,
    sender::DefaultTransactionSender,
};

pub async fn run(config: Config) -> Result<(), Box<dyn Error>> {
    tracing::info!(?config, "Starting the authority-claimer");

    // Creating the metrics and health server.
    let metrics = AuthorityClaimerMetrics::new();
    let http_server_handle =
        http_server::start(config.http_server_config, metrics.clone().into());

    let claimer_handle = {
        let config = config.authority_claimer_config;

        let dapp_address = config.dapp_address;
        let dapp_metadata = DAppMetadata {
            chain_id: config.tx_manager_config.chain_id,
            dapp_address,
        };

        // Creating the duplicate checker.
        trace!("Creating the duplicate checker");
        let duplicate_checker = DefaultDuplicateChecker::new()?;

        // Creating the transaction sender.
        trace!("Creating the transaction sender");
        let transaction_sender = DefaultTransactionSender::new(
            dapp_metadata.clone(),
            metrics.clone(),
        )?;

        // Creating the broker listener.
        trace!("Creating the broker listener");
        let broker_listener =
            DefaultBrokerListener::new(config.broker_config, dapp_metadata)
                .await?;

        // Creating the claimer.
        trace!("Creating the claimer");
        let claimer =
            AbstractClaimer::new(duplicate_checker, transaction_sender);

        // Returning the claimer event loop.
        broker_listener.start(claimer)
    };

    // Starting the HTTP server and the claimer loop.
    tokio::select! {
        ret = http_server_handle => { ret? }
        ret = claimer_handle     => { ret? }
    };

    unreachable!()
}

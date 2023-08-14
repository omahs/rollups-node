// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use async_trait::async_trait;
use rollups_events::{
    Broker, BrokerConfig, BrokerError, DAppMetadata, RollupsClaimsStream,
    INITIAL_ID,
};
use snafu::ResultExt;
use std::fmt::Debug;
use tracing::trace;

use crate::claimer::Claimer;

/// The `BrokerListener` starts a perpetual loop that listens for new claims from
/// the broker and sends them to be processed by the injected `Claimer`.
#[async_trait]
pub trait BrokerListener<C: Claimer> {
    type Error: snafu::Error + 'static;

    /// Starts the polling loop.
    async fn start(self, claimer: C) -> Result<(), Self::Error>;
}

// ------------------------------------------------------------------------------------------------
// DefaultBrokerListener
// ------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct DefaultBrokerListener {
    broker: Broker,
    stream: RollupsClaimsStream,
    last_claim_id: String,
}

#[derive(Debug, snafu::Snafu)]
pub enum BrokerListenerError<C: Claimer> {
    #[snafu(display("broker error"))]
    BrokerError { source: BrokerError },

    #[snafu(display("claimer error"))]
    ClaimerError { source: C::Error },
}

impl DefaultBrokerListener {
    pub async fn new(
        broker_config: BrokerConfig,
        dapp_metadata: DAppMetadata,
    ) -> Result<Self, BrokerError> {
        tracing::trace!("Connecting to the broker ({:?})", broker_config);
        let broker = Broker::new(broker_config).await?;
        let stream = RollupsClaimsStream::new(&dapp_metadata);
        let last_claim_id = INITIAL_ID.to_string();
        Ok(Self {
            broker,
            stream,
            last_claim_id,
        })
    }
}

#[async_trait]
impl<C> BrokerListener<C> for DefaultBrokerListener
where
    C: Claimer + Send + 'static,
{
    type Error = BrokerListenerError<C>;

    async fn start(mut self, mut claimer: C) -> Result<(), Self::Error> {
        trace!("Starting the event loop");
        loop {
            tracing::trace!("Waiting for claim with id {}", self.last_claim_id);
            let event = self
                .broker
                .consume_blocking(&self.stream, &self.last_claim_id)
                .await
                .context(BrokerSnafu)?;

            let rollups_claim = event.payload.clone();
            trace!("Got a claim from the broker: {:?}", rollups_claim);
            claimer = claimer
                .send_rollups_claim(rollups_claim)
                .await
                .context(ClaimerSnafu)?;
            tracing::trace!("Consumed event {:?}", event);

            self.last_claim_id = event.id;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use testcontainers::clients::Cli;

    use test_fixtures::BrokerFixture;

    use crate::{
        listener::{BrokerListener, DefaultBrokerListener},
        mock,
    };

    async fn setup(docker: &Cli) -> (BrokerFixture, DefaultBrokerListener) {
        mock::setup_broker(docker, false).await.unwrap()
    }

    #[tokio::test]
    async fn instantiate_new_broker_listener_ok() {
        let docker = Cli::default();
        let _ = setup(&docker).await;
    }

    #[tokio::test]
    async fn instantiate_new_broker_listener_error() {
        let docker = Cli::default();
        let result = mock::setup_broker(&docker, true).await;
        assert!(result.is_err(), "setup_broker didn't fail as it should");
        let error = result.err().unwrap().to_string();
        assert_eq!(error, "error connecting to Redis");
    }

    #[tokio::test]
    async fn start_broker_listener_with_claims_enqueued() {
        let docker = Cli::default();
        let (fixture, broker) = setup(&docker).await;
        let n = 5;
        let claimer = mock::Claimer::new(n);
        mock::produce_rollups_claims(&fixture, n, 0).await;
        mock::produce_last_claim(&fixture, n).await;
        let result = broker.start(claimer).await;
        mock::assert_broker_listener_ended(result);
    }

    #[tokio::test]
    async fn start_broker_listener_with_no_claims_enqueued() {
        let docker = Cli::default();
        let (fixture, broker) = setup(&docker).await;
        let n = 7;
        let claimer = mock::Claimer::new(n);

        let broker_listener = tokio::spawn(async move {
            println!("Spawned the broker-listener thread.");
            let result = broker.start(claimer).await;
            mock::assert_broker_listener_ended(result);
        });

        println!("Going to sleep for 1 second.");
        tokio::time::sleep(Duration::from_secs(1)).await;

        let x = 2;
        println!("Creating {} claims.", x);
        mock::produce_rollups_claims(&fixture, x, 0).await;

        println!("Going to sleep for 2 seconds.");
        tokio::time::sleep(Duration::from_secs(2)).await;

        let y = 5;
        println!("Creating {} claims.", y);
        mock::produce_rollups_claims(&fixture, y, x).await;

        assert_eq!(x + y, n);
        mock::produce_last_claim(&fixture, n).await;

        broker_listener.await.unwrap();
    }

    #[tokio::test]
    async fn start_broker_listener_and_fail_without_consuming_claims() {
        let docker = Cli::default();
        let (fixture, broker) = setup(&docker).await;
        let n = 0;
        let claimer = mock::Claimer::new_with_error(n);
        mock::produce_last_claim(&fixture, n).await;
        let result = broker.start(claimer).await;
        mock::assert_broker_listener_failed(result);
    }

    #[tokio::test]
    async fn start_broker_listener_and_fail_after_consuming_some_claims() {
        let docker = Cli::default();
        let (fixture, broker) = setup(&docker).await;
        let n = 5;
        let claimer = mock::Claimer::new_with_error(n);
        mock::produce_rollups_claims(&fixture, n, 0).await;
        mock::produce_last_claim(&fixture, n).await;
        let result = broker.start(claimer).await;
        mock::assert_broker_listener_failed(result);
    }
}

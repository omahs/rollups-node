use std::time::Duration;

use async_trait::async_trait;
use backoff::ExponentialBackoffBuilder;
use rollups_events::{
    BrokerConfig, BrokerEndpoint, BrokerError, DAppMetadata, RedactedUrl,
    RollupsClaim, Url,
};
use snafu::{OptionExt, Snafu};
use test_fixtures::BrokerFixture;
use testcontainers::clients::Cli;

use crate::{
    claimer,
    listener::{BrokerListenerError, DefaultBrokerListener},
};

#[derive(Debug)]
pub struct Claimer {
    results: Vec<Result<(), ClaimerError>>,
}

#[derive(Clone, Debug, Snafu)]
pub enum ClaimerError {
    EndError,
    InternalError,
    MockError,
}

impl Claimer {
    /// Creates a `Claimer` that proccesses `n` claims before returning
    /// the `ClaimerError::EndError` error.
    pub fn new(n: usize) -> Self {
        let mut results: Vec<Result<(), ClaimerError>> = vec![Ok(()); n];
        results.insert(0, Err(ClaimerError::EndError));
        Self { results }
    }

    /// Creates a `Claimer` that proccesses `n` claims before returning
    /// the `ClaimerError::MockError` error.
    pub fn new_with_error(n: usize) -> Self {
        let mut results: Vec<Result<(), ClaimerError>> = vec![Ok(()); n];
        results.insert(0, Err(ClaimerError::MockError));
        Self { results }
    }
}

#[async_trait]
impl claimer::Claimer for Claimer {
    type Error = ClaimerError;

    async fn send_rollups_claim(
        mut self,
        _: RollupsClaim,
    ) -> Result<Self, Self::Error> {
        let length = self.results.len() - 1;
        println!("The mock claimer is consuming claim {}.", length);
        self.results.pop().context(InternalSnafu)?.map(|_| self)
    }
}

pub fn assert_broker_listener_ended(
    result: Result<(), BrokerListenerError<Claimer>>,
) {
    assert!(result.is_err());
    match result {
        Ok(_) => panic!("broker listener returned with Ok(())"),
        Err(BrokerListenerError::ClaimerError { source }) => {
            assert_eq!(source.to_string(), ClaimerError::EndError.to_string())
        }
        Err(err) => panic!("broker listener failed with error {:?}", err),
    }
}

pub fn assert_broker_listener_failed(
    result: Result<(), BrokerListenerError<Claimer>>,
) {
    assert!(result.is_err());
    match result {
        Ok(_) => panic!("broker listener returned with Ok(())"),
        Err(BrokerListenerError::ClaimerError { source }) => {
            assert_eq!(source.to_string(), ClaimerError::MockError.to_string())
        }
        Err(err) => panic!("broker listener failed with error {:?}", err),
    }
}

pub async fn setup_broker(
    docker: &Cli,
    should_fail: bool,
) -> Result<(BrokerFixture, DefaultBrokerListener), BrokerError> {
    let fixture = BrokerFixture::setup(docker).await;

    let redis_endpoint = if should_fail {
        BrokerEndpoint::Single(RedactedUrl::new(
            Url::parse("https://invalid.com").unwrap(),
        ))
    } else {
        fixture.redis_endpoint().clone()
    };

    let config = BrokerConfig {
        redis_endpoint,
        consume_timeout: 300000,
        backoff: ExponentialBackoffBuilder::new()
            .with_initial_interval(Duration::from_millis(1000))
            .with_max_elapsed_time(Some(Duration::from_millis(3000)))
            .build(),
    };
    let metadata = DAppMetadata {
        chain_id: fixture.chain_id(),
        dapp_address: fixture.dapp_address().clone(),
    };
    let broker = DefaultBrokerListener::new(config, metadata).await?;
    Ok((fixture, broker))
}

pub async fn produce_rollups_claims(
    fixture: &BrokerFixture<'_>,
    n: usize,
    epoch_index_start: usize,
) -> Vec<RollupsClaim> {
    let mut rollups_claims = Vec::new();
    for i in 0..n {
        let mut rollups_claim = RollupsClaim::default();
        rollups_claim.epoch_index = (i + epoch_index_start) as u64;
        fixture.produce_rollups_claim(rollups_claim.clone()).await;
        rollups_claims.push(rollups_claim);
    }
    rollups_claims
}

/// The last claim should trigger the `ClaimerError::EndError` error.
pub async fn produce_last_claim(
    fixture: &BrokerFixture<'_>,
    epoch_index: usize,
) -> Vec<RollupsClaim> {
    produce_rollups_claims(fixture, 1, epoch_index).await
}

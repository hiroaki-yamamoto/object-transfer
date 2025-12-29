use ::std::fmt::{Debug, Display};

use ::async_nats::error::Error as NatsKindError;
use ::thiserror::Error;

use crate::errors::BrokerError;

/// Error type for NATS SubFetcher operations.
#[derive(Error, Debug)]
pub enum NatsSubFetcherError {
  /// Error during the creation of a JetStream stream.
  #[error("NATS JetStream Stream Creation Error: {0}")]
  JetStreamStreamCreation(
    #[from] async_nats::jetstream::context::CreateStreamError,
  ),
}

impl<T> From<NatsKindError<T>> for BrokerError
where
  T: Debug + Display + Clone + PartialEq + Send + Sync + 'static,
{
  fn from(e: NatsKindError<T>) -> Self {
    BrokerError::new(e)
  }
}

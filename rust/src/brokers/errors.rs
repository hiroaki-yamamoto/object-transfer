use ::std::error::Error as StdError;

use ::thiserror::Error;

/// Error type for broker-specific errors (for example, from NATS or JetStream)
/// that provides a unified error interface for the messaging system.
#[derive(Error, Debug)]
#[error(transparent)]
pub struct BrokerError(#[from] Box<dyn StdError + Send + Sync>);

impl BrokerError {
  /// Creates a new BrokerError from any error type that implements StdError.
  pub fn new<E>(err: E) -> Self
  where
    E: StdError + Send + Sync + 'static,
  {
    Self(Box::new(err))
  }
}

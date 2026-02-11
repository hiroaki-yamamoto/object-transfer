use ::thiserror::Error;

use super::broker::BrokerError;

/// Error type for unsubscribe operations in the messaging system.
#[derive(Error, Debug)]
pub enum UnSubError {
  /// Broker error.
  #[error("Broker error: {0}")]
  BrokerError(#[from] BrokerError),
  #[error("No Unsubscribe handler found")]
  NoHandler,
}

use ::thiserror::Error;

use super::broker::BrokerError;

/// Error type for acknowledgment operations in the messaging system.
#[derive(Error, Debug)]
pub enum AckError {
  /// Error during broker operations.
  #[error("Broker error: {0}")]
  BrokerError(#[from] BrokerError),
  /// Generic error variant for miscellaneous errors (Test use only).
  #[cfg(test)]
  #[error("Error Test")]
  ErrorTest,
}

use ::serde::ser::Error as EncErr;
use ::thiserror::Error;

use super::BrokerError;
use super::encode::EncodeError;

/// Error type for publishing operations in the messaging system.
#[derive(Error, Debug)]
pub enum PubError<EncodeErrorType: EncErr + Send + Sync> {
  /// Error during broker operations.
  #[error("Broker error: {0}")]
  BrokerError(#[from] BrokerError),
  #[error("Encoding error: {0}")]
  EncodeError(#[from] EncodeError<EncodeErrorType>),
  /// Generic error variant for miscellaneous errors (Test use only).
  #[cfg(test)]
  #[error("Error Test")]
  ErrorTest,
}

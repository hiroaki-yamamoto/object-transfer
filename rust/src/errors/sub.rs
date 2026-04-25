use ::serde::de::Error as DeErr;
use ::thiserror::Error;

use super::BrokerError;
use super::ack::AckError;
use super::decode::DecodeError;

/// Error type for subscription operations in the messaging system.
#[derive(Error, Debug)]
pub enum SubError<DecodeErrorType: DeErr + Send + Sync> {
  /// Broker error.
  #[error("Broker error: {0}")]
  BrokerError(#[from] BrokerError),
  /// Acknowledgment error.
  #[error("Acknowledgment error: {0}")]
  AckError(#[from] AckError),
  /// Decoding error for deserialization failures.
  #[error("Decoding error: {0}")]
  DecodeError(#[from] DecodeError<DecodeErrorType>),
  /// Generic error variant for miscellaneous errors (Test use only).
  #[cfg(test)]
  #[error("Error Test")]
  ErrorTest,
}

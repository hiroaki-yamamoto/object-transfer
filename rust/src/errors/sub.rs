use ::thiserror::Error;

use super::ack::AckError;
use super::broker::BrokerError;

/// Error type for subscription operations in the messaging system.
#[derive(Error, Debug)]
pub enum SubError {
  /// Broker error.
  #[error("Broker error: {0}")]
  BrokerError(#[from] BrokerError),
  /// Acknowledgment error.
  #[error("Acknowledgment error: {0}")]
  AckError(#[from] AckError),
  /// Error during message serialization or deserialization to/from JSON.
  #[error("JSON error: {0}")]
  Json(#[from] serde_json::Error),
  /// Error during message deserialization from MessagePack.
  #[error("MessagePack decode error: {0}")]
  MessagePackDecode(#[from] rmp_serde::decode::Error),
  /// Generic error variant for miscellaneous errors (Test use only).
  #[cfg(test)]
  #[error("Error Test")]
  ErrorTest,
}

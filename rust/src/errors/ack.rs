use ::thiserror::Error;

/// Error type for acknowledgment operations in the messaging system.
#[derive(Error, Debug)]
pub enum AckError {
  /// Error during acknowledgment from Nats.
  #[error("NATS error: {0}")]
  Nats(#[from] async_nats::Error),
  /// Generic error variant for miscellaneous errors (Test use only).
  #[cfg(test)]
  #[error("Error Test")]
  ErrorTest,
}

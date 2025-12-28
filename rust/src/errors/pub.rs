use ::thiserror::Error;

/// Error type for publishing operations in the messaging system.
#[derive(Error, Debug)]
pub enum PubError {
  /// Error during the publishing of a message to a Nats JetStream context.
  #[error("NATS JetStream publish error: {0}")]
  NatsPublish(#[from] async_nats::jetstream::context::PublishError),
  /// Error during message serialization or deserialization to/from JSON.
  #[error("JSON error: {0}")]
  Json(#[from] serde_json::Error),
  /// Error during message serialization to MessagePack.
  #[error("MessagePack encode error: {0}")]
  MessagePackEncode(#[from] rmp_serde::encode::Error),
  /// Generic error variant for miscellaneous errors (Test use only).
  #[cfg(test)]
  #[error("Error Test")]
  ErrorTest,
}

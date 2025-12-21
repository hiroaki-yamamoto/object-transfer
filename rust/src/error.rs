//! Error definitions shared across the crate.
//! Wraps NATS, JetStream, and serialization errors under a single type.

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

/// Error type for subscription operations in the messaging system.
#[derive(Error, Debug)]
pub enum SubError {
  /// Acknowledgment error
  #[error("Acknowledgment error: {0}")]
  AckError(#[from] AckError),
  /// Error originating from NATS or its JetStream components.
  #[error("NATS error: {0}")]
  Nats(#[from] async_nats::Error),
  /// Nats Streaming Error.
  #[error("NATS JetStream stream error: {0}")]
  NatsStream(#[from] async_nats::jetstream::consumer::StreamError),
  /// Error during the creation of a JetStream consumer.
  #[error("NATS JetStream Consumer Error: {0}")]
  NatsJetStreamConsumer(#[from] async_nats::jetstream::stream::ConsumerError),
  /// Errror retrieving messages from a NATS JetStream Pull Consumer.
  #[error("NATS JetStream Pull Consumer message error: {0}")]
  NatsPullMessage(
    #[from] async_nats::jetstream::consumer::pull::MessagesError,
  ),
  /// Error retrieving messages from a NATS JetStream Push Consumer.
  #[error("NATS JetStream Push Consumer message error: {0}")]
  NatsPushMessage(
    #[from] async_nats::jetstream::consumer::push::MessagesError,
  ),
  /// Error retrieving messages from a NATS JetStream Ordered Pull Consumer.
  #[error("NATS JetStream Ordered Pull Consumer message error: {0}")]
  NatsOrderedPullMessage(
    #[from] async_nats::jetstream::consumer::pull::OrderedError,
  ),
  /// Error retrieving messages from a NATS JetStream Ordered Push Consumer.
  #[error("NATS JetStream Push Consumer message error: {0}")]
  NatsOrderedPushMessage(
    #[from] async_nats::jetstream::consumer::push::OrderedError,
  ),
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

/// Error type for unsubscribe operations in the messaging system.
#[derive(Error, Debug)]
pub enum UnSubError {
  /// Nats Unsubscribe Error.
  #[error("NATS Unsubscribe error: {0}")]
  NatsUnsubscribe(#[from] async_nats::jetstream::stream::ConsumerError),
}

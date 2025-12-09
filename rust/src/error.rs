use ::thiserror::Error;

/// Centralized error type for the messaging system.
#[derive(Error, Debug)]
pub enum Error {
  /// Error originating from NATS or its JetStream components.
  #[error("NATS error: {0}")]
  Nats(#[from] async_nats::Error),
  #[error("NATS JetStream Stream Creation Error: {0}")]
  /// Error during the creation of a JetStream stream.
  JetStreamStreamCreation(
    #[from] async_nats::jetstream::context::CreateStreamError,
  ),
  /// Error during the creation of a JetStream consumer.
  #[error("NATS JetStream Consumer Error: {0}")]
  NatsJetStreamConsumer(#[from] async_nats::jetstream::stream::ConsumerError),
  /// Error during the publishing of a message to a Nats JetStream context.
  #[error("NATS JetStream publish error: {0}")]
  NatsPublish(#[from] async_nats::jetstream::context::PublishError),
  /// Nats Streaming Error.
  #[error("NATS JetStream stream error: {0}")]
  NatsStream(#[from] async_nats::jetstream::consumer::StreamError),
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
  /// Error during message serialization to MessagePack.
  #[error("MessagePack encode error: {0}")]
  MessagePackEncode(#[from] rmp_serde::encode::Error),
  /// Error during message deserialization from MessagePack.
  #[error("MessagePack decode error: {0}")]
  MessagePackDecode(#[from] rmp_serde::decode::Error),
  /// Generic error variant for miscellaneous errors (Test use only).
  #[cfg(test)]
  #[error("Error Test")]
  ErrorTest,
}

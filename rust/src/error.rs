use ::thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("NATS error: {0}")]
  Nats(#[from] async_nats::Error),
  #[error("NATS JetStream Stream Creation Error: {0}")]
  JetStreamStreamCreation(
    #[from] async_nats::jetstream::context::CreateStreamError,
  ),
  #[error("NATS JetStream Consumer Error: {0}")]
  NatsJetStreamConsumer(#[from] async_nats::jetstream::stream::ConsumerError),
  #[error("NATS JetStream publish error: {0}")]
  NatsPublish(#[from] async_nats::jetstream::context::PublishError),
  #[error("NATS JetStream stream error: {0}")]
  NatsStream(#[from] async_nats::jetstream::consumer::StreamError),
  #[error("NATS JetStream message error: {0}")]
  NatsMessage(#[from] async_nats::jetstream::consumer::pull::MessagesError),
  #[error("JSON error: {0}")]
  Json(#[from] serde_json::Error),
  #[error("MessagePack encode error: {0}")]
  MessagePackEncode(#[from] rmp_serde::encode::Error),
  #[error("MessagePack decode error: {0}")]
  MessagePackDecode(#[from] rmp_serde::decode::Error),
}

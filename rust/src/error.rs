use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("NATS error: {0}")]
  Nats(#[from] async_nats::Error),
  #[error("JetStream Stream Creation Error: {0}")]
  JetStreamStreamCreation(
    #[from] async_nats::jetstream::context::CreateStreamError,
  ),
  #[error("JetStream Consumer Error: {0}")]
  JetStreamConsumer(#[from] async_nats::jetstream::stream::ConsumerError),
  #[error("JetStream publish error: {0}")]
  Publish(#[from] async_nats::jetstream::context::PublishError),
  #[error("JetStream stream error: {0}")]
  Stream(#[from] async_nats::jetstream::consumer::StreamError),
  #[error("JSON error: {0}")]
  Json(#[from] serde_json::Error),
  #[error("MessagePack encode error: {0}")]
  MessagePackEncode(#[from] rmp_serde::encode::Error),
  #[error("MessagePack decode error: {0}")]
  MessagePackDecode(#[from] rmp_serde::decode::Error),
}

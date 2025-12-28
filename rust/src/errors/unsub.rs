use ::thiserror::Error;

/// Error type for unsubscribe operations in the messaging system.
#[derive(Error, Debug)]
pub enum UnSubError {
  /// Nats Unsubscribe Error.
  #[error("NATS Unsubscribe error: {0}")]
  NatsUnsubscribe(#[from] async_nats::jetstream::stream::ConsumerError),
}

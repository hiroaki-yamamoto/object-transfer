use ::thiserror::Error;

/// Error type for NATS SubFetcher operations.
#[derive(Error, Debug)]
pub enum NatsSubFetcherError {
  /// Error during the creation of a JetStream stream.
  #[error("NATS JetStream Stream Creation Error: {0}")]
  JetStreamStreamCreation(
    #[from] async_nats::jetstream::context::CreateStreamError,
  ),
}

/// Configuration for the Redis publisher.
///
/// This struct holds the configuration settings needed to initialize and manage
/// a Redis publisher instance, including the consumer group name for Redis streams.
#[derive(Clone, Debug)]
pub struct PublisherConfig {
  pub(in super::super) group_name: Option<String>,
  pub(in super::super) stream_length: usize,
}

const MAX_STREAM_LENGTH_DEFAULT: usize = 500_000;

impl PublisherConfig {
  /// Creates a new `PublisherConfig` instance.
  ///
  /// # Returns
  ///
  /// A new `PublisherConfig` instance with the specified group name.
  ///
  /// # Default values
  /// - `group_name`: `None`, which defaults to the stream name.
  /// - `stream_length`: up to 500,000 messages.
  pub fn new() -> Self {
    Self {
      group_name: None,
      stream_length: MAX_STREAM_LENGTH_DEFAULT,
    }
  }

  /// Sets the Redis stream consumer group name used by the publisher.
  pub fn group_name(mut self, group_name: impl Into<String>) -> Self {
    self.group_name = Some(group_name.into());
    self
  }

  /// Sets the maximum length of the Redis stream.
  pub fn stream_length(mut self, length: usize) -> Self {
    self.stream_length = length;
    self
  }
}

/// Configuration for the Redis publisher.
///
/// This struct holds the configuration settings needed to initialize and manage
/// a Redis publisher instance, including the consumer group name for Redis streams.
pub struct PublisherConfig {
  pub(super) group_name: String,
}

impl PublisherConfig {
  /// Creates a new `PublisherConfig` instance.
  ///
  /// # Arguments
  ///
  /// * `group_name` - The name of the consumer group for Redis streams.
  ///
  /// # Returns
  ///
  /// A new `PublisherConfig` instance with the specified group name.
  pub fn new(group_name: &str) -> Self {
    return Self {
      group_name: group_name.to_string(),
    };
  }
}

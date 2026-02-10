/// Configuration for Redis stream subscribers, including fetch count and block time.
///
/// Defaults:
/// - `consumer_name`: same as `topic_name`
/// - `group_name`: same as `topic_name`
/// - `topic_name`: as provided
/// - `num_fetch`: 10
/// - `block_time`: 5000 ms
#[derive(Clone, Debug)]
pub struct SubscriberConfig {
  pub(in super::super) consumer_name: String,
  pub(in super::super) group_name: String,
  pub(in super::super) topic_name: String,
  pub(in super::super) num_fetch: usize,
  pub(in super::super) block_time: usize,
  pub(in super::super) auto_claim: bool,
}

impl SubscriberConfig {
  /// Creates a new configuration with defaults and the given topic name.
  /// # Parameters
  ///
  /// * `topic_name` - The name of the Redis stream topic.
  ///
  pub fn new(topic_name: impl Into<String>) -> Self {
    let topic_name = topic_name.into();
    return Self {
      consumer_name: topic_name.clone(),
      group_name: topic_name.clone(),
      topic_name: topic_name,
      num_fetch: 10,    // Default number to fetch
      block_time: 5000, // Default block time in milliseconds (5 seconds)
      auto_claim: true, // Default to auto-claim pending messages
    };
  }

  /// Sets the consumer name.
  pub fn consumer_name(mut self, consumer_name: impl Into<String>) -> Self {
    self.consumer_name = consumer_name.into();
    self
  }

  /// Sets the consumer group name.
  pub fn group_name(mut self, group_name: impl Into<String>) -> Self {
    self.group_name = group_name.into();
    self
  }

  /// Sets the topic (stream) name.
  pub fn topic_name(mut self, topic_name: impl Into<String>) -> Self {
    self.topic_name = topic_name.into();
    self
  }

  /// Sets how many messages to fetch per read.
  pub fn num_fetch(mut self, count: usize) -> Self {
    self.num_fetch = count;
    self
  }

  /// Sets the blocking time in milliseconds for each read.
  pub fn block_time(mut self, millis: usize) -> Self {
    self.block_time = millis;
    self
  }

  /// Sets whether the session to send auto-claim for pending messages.
  pub fn auto_claim(mut self, auto_claim: bool) -> Self {
    self.auto_claim = auto_claim;
    self
  }
}

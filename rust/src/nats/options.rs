use ::std::sync::Arc;

use ::async_nats::jetstream::{
  consumer::pull::Config as PullConfig, stream::Config as StreamConfig,
};

use crate::r#enum::Format;

/// Configuration options for creating an acknowledgment-based subscriber.
///
/// This struct provides a builder pattern for configuring NATS JetStream
/// consumers with pull-based message consumption and automatic acknowledgment options.
#[derive(Debug)]
pub struct AckSubOptions {
  pub(super) stream_cfg: StreamConfig,
  pub(super) pull_cfg: PullConfig,
  pub(super) auto_ack: bool,
  pub(super) format: Format,
}

impl AckSubOptions {
  /// Creates a new `AckSubOptions` with the specified format and name.
  ///
  /// # Arguments
  /// * `format` - The message format to use for serialization/deserialization
  /// * `name` - The name for both the stream and consumer
  ///
  /// # Returns
  /// A new `AckSubOptions` instance with default settings and auto-acknowledgment enabled
  pub fn new(format: Format, name: Arc<str>) -> Self {
    Self {
      stream_cfg: StreamConfig {
        name: name.clone().to_string(),
        ..Default::default()
      },
      pull_cfg: PullConfig {
        name: Some(name.clone().to_string()),
        ..PullConfig::default()
      },
      auto_ack: true,
      format,
    }
  }

  /// Sets whether messages should be automatically acknowledged.
  ///
  /// # Arguments
  /// * `auto_ack` - If true, messages will be automatically acknowledged after processing
  ///
  /// # Returns
  /// Self for method chaining
  pub fn auto_ack(mut self, auto_ack: bool) -> Self {
    self.auto_ack = auto_ack;
    return self;
  }

  /// Sets the stream name.
  ///
  /// # Arguments
  /// * `name` - The name to assign to the stream
  ///
  /// # Returns
  /// Self for method chaining
  pub fn name(mut self, name: impl Into<String>) -> Self {
    self.stream_cfg.name = name.into();
    self
  }

  /// Sets the subjects that the stream should listen to.
  ///
  /// # Arguments
  /// * `subjects` - A vector of subject patterns to subscribe to
  ///
  /// # Returns
  /// Self for method chaining
  pub fn subjects(mut self, subjects: Vec<impl Into<String>>) -> Self {
    self.stream_cfg.subjects = subjects.into_iter().map(Into::into).collect();
    self
  }

  /// Sets the durable name for the consumer.
  ///
  /// A durable consumer will persist its state and can resume consumption
  /// after disconnection.
  ///
  /// # Arguments
  /// * `durable_name` - The durable name for the consumer
  ///
  /// # Returns
  /// Self for method chaining
  pub fn durable_name(mut self, durable_name: impl Into<String>) -> Self {
    self.pull_cfg.durable_name = Some(durable_name.into());
    self
  }

  /// Sets the message format for serialization/deserialization.
  ///
  /// # Arguments
  /// * `format` - The format to use for message encoding
  ///
  /// # Returns
  /// Self for method chaining
  pub fn format(mut self, format: Format) -> Self {
    self.format = format;
    self
  }

  /// Sets the complete stream configuration.
  ///
  /// This replaces the entire stream configuration with the provided one.
  ///
  /// # Arguments
  /// * `stream_cfg` - The stream configuration to use
  ///
  /// # Returns
  /// Self for method chaining
  pub fn stream_config(mut self, stream_cfg: StreamConfig) -> Self {
    self.stream_cfg = stream_cfg;
    self
  }

  /// Sets the complete pull consumer configuration.
  ///
  /// This replaces the entire pull consumer configuration with the provided one.
  ///
  /// # Arguments
  /// * `pull_cfg` - The pull consumer configuration to use
  ///
  /// # Returns
  /// Self for method chaining
  pub fn pull_config(mut self, pull_cfg: PullConfig) -> Self {
    self.pull_cfg = pull_cfg;
    self
  }
}

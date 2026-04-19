//! Configuration options for subscriptions.
//!
//! This module provides the `SubOpt` struct for configuring subscriber behavior.
//! It allows control over automatic acknowledgment handling and message format
//! selection for subscription operations.

use crate::format::Format;

#[derive(Debug, Clone)]
pub struct SubOpt {
  pub(crate) auto_ack: bool,
  pub(crate) format: Format,
}

impl Default for SubOpt {
  fn default() -> Self {
    Self {
      auto_ack: true,
      format: Format::JSON,
    }
  }
}

impl SubOpt {
  /// Creates a new `SubOpt` with the specified format and auto-acknowledgment setting.
  ///
  /// # Returns
  /// A new `SubOpt` instance with the specified settings
  pub fn new() -> Self {
    Self::default()
  }

  /// Sets whether messages should be automatically acknowledged.
  ///
  /// # Arguments
  /// * `auto_ack` - If true, messages will be automatically acknowledged after processing
  ///
  /// # Returns
  /// The updated `SubOpt` instance
  pub fn auto_ack(mut self, auto_ack: bool) -> Self {
    self.auto_ack = auto_ack;
    self
  }

  /// Sets the message format for serialization/deserialization.
  ///
  /// # Arguments
  /// * `format` - The message format to use
  ///
  /// # Returns
  /// The updated `SubOpt` instance
  pub fn format(mut self, format: Format) -> Self {
    self.format = format;
    self
  }
}

//! Configuration options for subscriptions.
//!
//! This module provides the `SubOpt` struct for configuring subscriber behavior.
//! It allows control over automatic acknowledgment handling and message format
//! selection for subscription operations.

use ::core::fmt::Debug;

#[derive(Debug, Clone)]
pub struct SubOpt {
  pub(crate) auto_ack: bool,
}

impl Default for SubOpt {
  fn default() -> Self {
    Self { auto_ack: true }
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
}

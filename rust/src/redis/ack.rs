//! Redis-based acknowledgment implementation for stream messages.
//!
//! This module provides the [`Ack`] struct, which handles acknowledgment of messages
//! consumed from Redis streams within a consumer group.

use ::async_trait::async_trait;
use ::futures::TryFutureExt;
use ::redis::AsyncTypedCommands;
use ::redis::aio::MultiplexedConnection;

use crate::errors::{AckError, BrokerError};
use crate::traits::AckTrait;

use super::errors::AckError as RedisAckError;

/// Represents an acknowledgment for a message in a Redis stream consumer group.
///
/// The `Ack` struct is responsible for acknowledging a message that has been
/// successfully processed by a consumer in a Redis stream. It maintains a connection
/// to the Redis instance and stores the necessary identifiers to track which message
/// should be acknowledged.
#[derive(Clone)]
pub struct Ack {
  group: String,
  stream_name: String,
  id: String,
  con: MultiplexedConnection,
}

impl Ack {
  /// Creates a new `Ack` instance for acknowledging a Redis stream message.
  ///
  /// # Arguments
  ///
  /// * `con` - A reference to the multiplexed Redis connection
  /// * `group` - The consumer group name
  /// * `stream_name` - The name of the Redis stream
  /// * `id` - The unique identifier of the message to acknowledge
  ///
  /// # Returns
  ///
  /// A new `Ack` instance configured with the provided parameters.
  pub(super) fn new(
    con: &MultiplexedConnection,
    group: impl Into<String>,
    stream_name: impl Into<String>,
    id: impl Into<String>,
  ) -> Self {
    Self {
      con: con.clone(),
      group: group.into(),
      stream_name: stream_name.into(),
      id: id.into(),
    }
  }
}

#[async_trait]
impl AckTrait for Ack {
  /// Acknowledges a message in the Redis stream consumer group.
  ///
  /// This method marks the message with the stored ID as processed within the
  /// consumer group. Once acknowledged, the message will no longer be pending
  /// for the consumer group.
  ///
  /// # Returns
  ///
  /// * `Ok(())` - If the acknowledgment was successful
  /// * `Err(AckError)` - If the acknowledgment operation failed (e.g., connection error)
  async fn ack(&self) -> Result<(), AckError> {
    let mut con = self.con.clone();
    con
      .xack(&self.stream_name, &self.group, &[&self.id])
      .map_err(|err| BrokerError::from(RedisAckError(err)))
      .await?;
    Ok(())
  }
}

use ::redis::RedisError;
use ::thiserror::Error;

use crate::errors::BrokerError;

/// Errors that can occur during Redis publish operations.
#[derive(Error, Debug)]
#[error("Redis publish error: {0}")]
pub enum PublishError {
  /// Error that occurs when creating a consumer group in Redis fails.
  #[error("Group Creation Error: {0}")]
  GroupCreation(RedisError),
  /// Error that occurs when pushing a message to a Redis stream fails.
  #[error("Message Pushing Error: {0}")]
  Push(RedisError),
}

impl From<PublishError> for BrokerError {
  fn from(err: PublishError) -> Self {
    BrokerError::new(err)
  }
}

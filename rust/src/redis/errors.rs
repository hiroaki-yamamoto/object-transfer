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

/// Errors that can occur during Redis subscribe operations.
#[derive(Error, Debug)]
#[error("Redis subscribe error: {0}")]
pub enum SubscribeError {
  /// Error that occurs when creating a consumer group in Redis fails.
  #[error("Group Creation Error: {0}")]
  GroupCreation(RedisError),
  /// Error that occurs when reading messages from a Redis stream fails.
  #[error("Message Reading Error: {0}")]
  Read(RedisError),
}

impl From<SubscribeError> for BrokerError {
  fn from(err: SubscribeError) -> Self {
    BrokerError::new(err)
  }
}

#[derive(Error, Debug)]
#[error("Redis acknowledgment error: {0}")]
pub struct AckError(pub RedisError);

impl From<AckError> for BrokerError {
  fn from(err: AckError) -> Self {
    BrokerError::new(err)
  }
}

#[derive(Error, Debug)]
#[error("Redis unsubscription error: {0}")]
pub struct UnsubscribeError(pub RedisError);

impl From<UnsubscribeError> for BrokerError {
  fn from(err: UnsubscribeError) -> Self {
    BrokerError::new(err)
  }
}

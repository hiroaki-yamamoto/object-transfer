use ::redis::RedisError;
use ::thiserror::Error;

use crate::errors::BrokerError;

#[derive(Error, Debug)]
#[error("Redis publish error: {0}")]
pub enum PublishError {
  #[error("Group Creation Error: {0}")]
  GroupCreation(RedisError),
  #[error("Message Pushing Error: {0}")]
  Push(RedisError),
}

impl From<PublishError> for BrokerError {
  fn from(err: PublishError) -> Self {
    BrokerError::new(err)
  }
}

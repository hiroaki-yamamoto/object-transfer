use ::redis::RedisError;
use ::thiserror::Error;

use crate::errors::BrokerError;

#[derive(Error, Debug)]
#[error("Redis publish error: {0}")]
pub struct PublishError(pub RedisError);

impl From<PublishError> for BrokerError {
  fn from(err: PublishError) -> Self {
    BrokerError::new(err)
  }
}

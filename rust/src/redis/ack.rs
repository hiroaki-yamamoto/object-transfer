use ::async_trait::async_trait;
use ::futures::TryFutureExt;
use ::redis::AsyncTypedCommands;
use ::redis::aio::MultiplexedConnection;

use crate::errors::{AckError, BrokerError};
use crate::traits::AckTrait;

use super::errors::AckError as RedisAckError;

pub struct Ack {
  group: String,
  stream_name: String,
  id: String,
  con: MultiplexedConnection,
}

impl Ack {
  pub(super) fn new(
    con: &MultiplexedConnection,
    group: String,
    stream_name: String,
    id: String,
  ) -> Self {
    Self {
      con: con.clone(),
      group,
      stream_name,
      id,
    }
  }
}

#[async_trait]
impl AckTrait for Ack {
  async fn ack(&self) -> Result<(), AckError> {
    let mut con = self.con.clone();
    con
      .xack(&self.stream_name, &self.group, &[&self.id])
      .map_err(|err| BrokerError::from(RedisAckError(err)))
      .await?;
    Ok(())
  }
}

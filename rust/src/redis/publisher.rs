use ::async_trait::async_trait;
use ::bytes::Bytes;
use ::futures::TryFutureExt;
use ::redis::{
  AsyncTypedCommands, aio::MultiplexedConnection, streams::StreamMaxlen,
};

use crate::errors::{BrokerError, PubError};
use crate::traits::PubCtxTrait;

use super::PublisherConfig;
use super::errors::PublishError;

pub struct Publisher {
  con: MultiplexedConnection,
  cfg: PublisherConfig,
}

impl Publisher {
  pub fn new(con: &MultiplexedConnection, cfg: PublisherConfig) -> Self {
    Self {
      con: con.clone(),
      cfg,
    }
  }
}

#[async_trait]
impl PubCtxTrait for Publisher {
  async fn publish(
    &self,
    topic: &str,
    payload: Bytes,
  ) -> Result<(), PubError> {
    let group_name = self
      .cfg
      .group_name
      .as_ref()
      .unwrap_or(&topic.to_string())
      .clone();
    let mut con = self.con.clone();
    con
      .xgroup_create(topic, group_name, "$ MKSTREAM")
      .map_err(|err| BrokerError::from(PublishError::GroupCreation(err)))
      .await?;
    con
      .xadd_maxlen(
        topic,
        StreamMaxlen::Approx(self.cfg.stream_length),
        "*",
        &[("data", payload.to_vec())],
      )
      .map_err(|err| BrokerError::from(PublishError::Push(err)))
      .await?;
    Ok(())
  }
}

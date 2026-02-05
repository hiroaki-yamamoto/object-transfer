use ::bytes::Bytes;
use ::futures::TryFutureExt;
use ::redis::{AsyncTypedCommands, aio::MultiplexedConnection};

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

impl PubCtxTrait for Publisher {
  async fn publish(
    &self,
    topic: &str,
    payload: Bytes,
  ) -> Result<(), PubError> {
    let mut con = self.con.clone();
    con
      .xgroup_create(topic, &self.cfg.group_name, "$ MKSTREAM")
      .map_err(|err| BrokerError::from(PublishError(err)))
      .await?;
    Ok(())
  }
}

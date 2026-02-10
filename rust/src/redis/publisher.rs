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

/// A Redis-based message publisher that sends messages to Redis streams.
///
/// The `Publisher` struct provides functionality to publish messages to Redis streams
/// with support for consumer groups and stream size management. It implements the
/// `PubCtxTrait` to provide a consistent interface for publishing operations.
#[derive(Clone)]
pub struct Publisher {
  /// The multiplexed Redis connection used for publishing messages.
  con: MultiplexedConnection,
  /// Configuration settings for the publisher, including group name and stream length.
  cfg: PublisherConfig,
}

impl Publisher {
  /// Creates a new `Publisher` instance.
  ///
  /// # Arguments
  ///
  /// * `con` - A reference to a multiplexed Redis connection.
  /// * `cfg` - Publisher configuration settings.
  ///
  /// # Returns
  ///
  /// A new `Publisher` instance configured with the provided connection and settings.
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
    if let Err(err) = con.xgroup_create_mkstream(topic, group_name, "$").await
    {
      // Ignore "BUSYGROUP" errors (group already exists) to make subscription idempotent.
      if err.code() != Some("BUSYGROUP") {
        return Err(
          BrokerError::from(PublishError::GroupCreation(err)).into(),
        );
      }
    }
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

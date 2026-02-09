use ::std::sync::Arc;

use ::async_stream::try_stream;
use ::async_trait::async_trait;
use ::bytes::Bytes;
use ::futures::TryFutureExt;
use ::futures::stream::BoxStream;
use ::redis::AsyncCommands;
use ::redis::Value;
use ::redis::aio::MultiplexedConnection;
use ::redis::streams::{
  StreamId, StreamKey, StreamReadOptions, StreamReadReply,
};

use crate::errors::{BrokerError, SubError};
use crate::traits::{AckTrait, SubCtxTrait};

use super::ack::Ack;
use super::config::SubscriberConfig;
use super::errors::SubscribeError;

/// A Redis-based message subscriber that handles subscription to Redis streams.
///
/// This struct manages subscription to Redis stream topics using consumer groups,
/// allowing for acknowledgment of processed messages and reliable message delivery.
#[derive(Clone)]
pub struct Subscriber {
  con: MultiplexedConnection,
  cfg: SubscriberConfig,
}

impl Subscriber {
  /// Creates a new subscriber instance.
  ///
  /// # Arguments
  ///
  /// * `con` - A reference to a multiplexed Redis connection
  /// * `cfg` - The subscriber configuration containing topic, group, and consumer names
  ///
  /// # Returns
  ///
  /// A new `Subscriber` instance configured with the provided connection and settings.
  pub fn new(con: &MultiplexedConnection, cfg: SubscriberConfig) -> Self {
    Self {
      con: con.clone(),
      cfg,
    }
  }
}

#[async_trait]
impl SubCtxTrait for Subscriber {
  /// Subscribes to a Redis stream and returns a stream of messages.
  ///
  /// Creates a consumer group if it doesn't exist, then continuously reads messages
  /// from the configured Redis stream. Each message is wrapped with an acknowledgment handler.
  ///
  /// # Returns
  ///
  /// A `Result` containing:
  /// - `Ok`: A boxed stream yielding tuples of (message bytes, acknowledgment handler)
  /// - `Err`: A `SubError` if subscription fails
  ///
  /// # Errors
  ///
  /// Returns an error if:
  /// - Consumer group creation fails
  /// - Stream reading operation fails
  async fn subscribe(
    &self,
  ) -> Result<
    BoxStream<Result<(Bytes, Arc<dyn AckTrait + Send + Sync>), SubError>>,
    SubError,
  > {
    let cfg = self.cfg.clone();
    let mut con = self.con.clone();
    con
      .xgroup_create_mkstream::<_, _, _, ()>(
        &cfg.topic_name,
        &cfg.group_name,
        "$",
      )
      .map_err(|err| BrokerError::from(SubscribeError::GroupCreation(err)))
      .await?;
    let opts = StreamReadOptions::default()
      .group(&cfg.group_name, &cfg.consumer_name)
      .count(cfg.num_fetch)
      .block(cfg.block_time);
    let stream = try_stream! {
        loop {
          let reply: StreamReadReply = con.xread_options(
            &[&cfg.topic_name], &[">"], &opts
          ).map_err(|err| BrokerError::from(SubscribeError::Read(err))).await?;
          for StreamKey { key, ids } in reply.keys {
            let _ = key;
            for StreamId {id, map, ..} in ids {
              for (_, value) in map {
                if let Value::BulkString(data) = value {
                  let payload = Bytes::from(data);
                  let ack = Arc::new(Ack::new(&self.con, &cfg.group_name, &cfg.topic_name, &id));
                  yield (payload, ack as Arc<dyn AckTrait + Send + Sync>);
                } else {
                  continue;
                }
              }
            }
          }
      }
    };
    return Ok(Box::pin(stream));
  }
}

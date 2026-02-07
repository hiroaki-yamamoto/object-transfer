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

use super::config::SubscriberConfig;
use super::errors::SubscribeError;

#[derive(Clone)]
pub struct Subscriber {
  con: MultiplexedConnection,
  cfg: SubscriberConfig,
}

impl Subscriber {
  pub fn new(con: &MultiplexedConnection, cfg: SubscriberConfig) -> Self {
    Self {
      con: con.clone(),
      cfg,
    }
  }
}

#[async_trait]
impl SubCtxTrait for Subscriber {
  async fn subscribe(
    &self,
  ) -> Result<
    BoxStream<Result<(Bytes, Arc<dyn AckTrait + Send + Sync>), SubError>>,
    SubError,
  > {
    let cfg = self.cfg.clone();
    self
      .con
      .xgroup_create_mkstream(cfg.topic_name, cfg.group_name, "$")
      .map_err(|err| BrokerError::from(SubscribeError::GroupCreation(err)))
      .await?;
    let opts = StreamReadOptions::default()
      .group(cfg.group_name, cfg.consumer_name)
      .count(cfg.num_fetch)
      .block(cfg.block_time);
    let stream = try_stream! {
        loop {
          let reply: StreamReadReply = self.con.xread_options(
            &[cfg.topic_name], &[">"], &opts
          ).map_err(|err| BrokerError::from(SubscribeError::Read(err))).await?;
          for StreamKey { key, ids } in reply.keys {
            for StreamId {id, map, ..} in ids {
              for (field, value) in map {
                if let Value::BulkString(data) = value {
                  let payload = Bytes::from(data);
                  let ack = Arc::new(super::ack::RedisAck {
                    con: self.con.clone(),
                    topic: key.clone(),
                    group: cfg.group_name.clone(),
                    consumer: cfg.consumer_name.clone(),
                    id: id.clone(),
                  });
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

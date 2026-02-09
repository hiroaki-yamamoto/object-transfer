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

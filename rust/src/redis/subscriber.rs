use ::std::sync::Arc;

use ::async_stream::{stream, try_stream};
use ::async_trait::async_trait;
use ::bytes::Bytes;
use ::futures::stream::BoxStream;
use ::futures::{StreamExt, TryFutureExt};
use ::redis::AsyncCommands;
use ::redis::Value;
use ::redis::aio::MultiplexedConnection;
use ::redis::streams::{
  StreamAutoClaimOptions, StreamAutoClaimReply, StreamId, StreamKey,
  StreamReadOptions, StreamReadReply,
};

use crate::errors::{BrokerError, SubError, UnSubError};
use crate::traits::{AckTrait, SubCtxTrait, UnSubTrait};

use super::ack::Ack;
use super::config::SubscriberConfig;
use super::errors::{SubscribeError, UnsubscribeError};

enum ReplyType {
  Read(StreamReadReply),
  AutoClaim(StreamAutoClaimReply),
  None,
}

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

  fn handle_stream_ids(
    &self,
    stream_ids: impl IntoIterator<Item = StreamId> + Send + Sync,
  ) -> Vec<(Bytes, Arc<dyn AckTrait + Send + Sync>)> {
    let mut results = Vec::new();
    let cfg = &self.cfg;
    for StreamId { id, map, .. } in stream_ids {
      for (_, value) in map {
        if let Value::BulkString(data) = value {
          let payload = Bytes::from(data);
          let ack = Arc::new(Ack::new(
            &self.con,
            &cfg.group_name,
            &cfg.topic_name,
            &id,
          ));
          results.push((payload, ack as Arc<dyn AckTrait + Send + Sync>));
        } else {
          continue;
        }
      }
    }
    results
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
    let mut con = self.con.clone();
    let cfg = &self.cfg;
    if let Err(err) = con
      .xgroup_create_mkstream::<_, _, _, ()>(
        &self.cfg.topic_name,
        &self.cfg.group_name,
        "$",
      )
      .await
    {
      // Ignore "BUSYGROUP" errors (group already exists) to make
      //   subscription idempotent.
      if err.code() != Some("BUSYGROUP") {
        return Err(
          BrokerError::from(SubscribeError::GroupCreation(err)).into(),
        );
      }
    }
    let opts = StreamReadOptions::default()
      .group(&cfg.group_name, &cfg.consumer_name)
      .count(cfg.num_fetch.clone())
      .block(cfg.block_time.clone());
    let stream = try_stream! {
        loop {
          let autoclaim = async {
            if cfg.auto_claim > 0 {
              let mut con = con.clone();
              con.xautoclaim_options(
                &cfg.topic_name,
                &cfg.group_name,
                &cfg.consumer_name,
                &cfg.auto_claim,
                "0-0",
                StreamAutoClaimOptions::default().count(cfg.num_fetch.clone()),
              ).map_err(
                |err| BrokerError::from(SubscribeError::AutoClaim(err))
              ).map_ok(|reply: StreamAutoClaimReply| reply.claimed).await
            } else {
              Ok(Vec::new())
            }
          };
          let stream_reply = async {
            let mut con = con.clone();
            con.xread_options(&[&cfg.topic_name], &[">"], &opts)
              .map_err(|err| BrokerError::from(SubscribeError::Read(err)))
              .map_ok(|reply: StreamReadReply| {
                let ids: Vec<StreamId> = reply.keys
                  .into_iter()
                  .flat_map(|k| k.ids)
                  .collect();
                ids
              })
              .await
          };
          let (auto_claimed_ids, read_ids) = futures::join!(autoclaim, stream_reply);
          let mut all_ids = Vec::new();
          if let Ok(mut ids) = auto_claimed_ids {
            all_ids.append(&mut ids);
          }
          if let Ok(mut ids) = read_ids {
            all_ids.append(&mut ids);
          }
          let values = self.handle_stream_ids(all_ids);
          for value in values {
            yield value;
          }
      }
    };
    Ok(Box::pin(stream))
  }
}

#[async_trait]
impl UnSubTrait for Subscriber {
  /// Unsubscribes from the Redis stream.
  ///
  /// This implementation is a no-op since Redis streams do not require explicit unsubscription.
  ///
  /// # Returns
  ///
  /// A `Result` indicating success or failure of the unsubscription operation.
  async fn unsubscribe(&self) -> Result<(), UnSubError> {
    let mut con = self.con.clone();
    let cfg = &self.cfg;
    let _: i32 = con
      .xgroup_delconsumer::<_, _, _, _>(
        &cfg.topic_name,
        &cfg.group_name,
        &cfg.consumer_name,
      )
      .map_err(|err| BrokerError::from(UnsubscribeError(err)))
      .await?;
    Ok(())
  }
}

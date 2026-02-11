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
  StreamAutoClaimOptions, StreamAutoClaimReply, StreamId, StreamReadOptions,
  StreamReadReply,
};

use crate::errors::{BrokerError, SubError, UnSubError};
use crate::traits::{AckTrait, SubCtxTrait, UnSubTrait};

use super::ack::Ack;
use super::config::SubscriberConfig;
use super::errors::{SubscribeError, UnsubscribeError};
use super::group_make::make_stream_group;

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
      if let Some(Value::BulkString(data)) = map.get("data") {
        let payload = Bytes::from(data.clone());
        let ack =
          Arc::new(Ack::new(&self.con, &cfg.group_name, &cfg.topic_name, &id));
        results.push((payload, ack as Arc<dyn AckTrait + Send + Sync>));
      } else {
        continue;
      }
    }
    results
  }

  async fn autoclaim(
    &self,
    autoclaim_id: impl Into<String>,
  ) -> Result<(Vec<StreamId>, String), BrokerError> {
    let id = autoclaim_id.into();
    let mut con = self.con.clone();
    let cfg = &self.cfg;
    if cfg.auto_claim > 0 {
      let reply: StreamAutoClaimReply = con
        .xautoclaim_options(
          &cfg.topic_name,
          &cfg.group_name,
          &cfg.consumer_name,
          &cfg.auto_claim,
          &id,
          StreamAutoClaimOptions::default().count(cfg.num_fetch.clone()),
        )
        .map_err(|err| BrokerError::from(SubscribeError::AutoClaim(err)))
        .await?;
      Ok((reply.claimed, reply.next_stream_id))
    } else {
      Ok((Vec::new(), id))
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
    let con = self.con.clone();
    let cfg = &self.cfg;
    make_stream_group(con.clone(), &cfg.topic_name, &cfg.group_name)
      .map_err(|err| BrokerError::from(SubscribeError::GroupCreation(err)))
      .await?;
    let opts = StreamReadOptions::default()
      .group(&cfg.group_name, &cfg.consumer_name)
      .count(cfg.num_fetch.clone())
      .block(cfg.block_time.clone());
    let stream = try_stream! {
      let mut autoclaim_id: String = "0-0".into();
      loop {
        let autoclaim = self.autoclaim(autoclaim_id.clone());
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
        let run_result = futures::try_join!(autoclaim, stream_reply);
        let mut all_ids = Vec::new();
        let ((mut auto, id), mut read) = run_result?;
        autoclaim_id = id;
        all_ids.append(&mut auto);
        all_ids.append(&mut read);
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
  /// Unsubscribes this consumer from the Redis stream consumer group.
  ///
  /// This implementation issues an `XGROUP DELCONSUMER` command to remove the
  /// configured consumer from the consumer group on the configured stream.
  /// While Redis streams do not require an explicit "unsubscribe" to stop
  /// receiving messages, this cleanup helps remove the consumer's pending
  /// entries from the group and free related server-side state.
  ///
  /// # Returns
  ///
  /// A `Result` indicating success or failure of the unsubscription (consumer
  /// cleanup) operation. Underlying Redis errors are wrapped in `UnSubError`.
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

use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;
use futures::TryStreamExt;
use futures::stream::BoxStream;
use serde::de::DeserializeOwned;

use crate::r#enum::Format;
use crate::errors::{SubError, UnSubError};
use crate::traits::{
  AckTrait, SubCtxTrait, SubOptTrait, SubTrait, UnSubTrait,
};

/// Subscriber wrapper that deserializes messages and optionally acknowledges
/// them.
///
/// The subscriber relies on a [`SubCtxTrait`] implementation for message
/// retrieval and a [`SubOptTrait`] provider for decoding and acknowledgment
/// behavior.
///
/// # Example
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use futures::StreamExt;
/// use serde::Deserialize;
/// use object_transfer::{Format, Sub};
/// use object_transfer::nats::{AckSubOptions, SubFetcher};
/// use object_transfer::traits::{SubTrait, UnSubTrait};
///
/// #[derive(Deserialize, Debug)]
/// struct Event {
///   id: u64,
///   name: String,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///   // Build a JetStream context and configure a durable pull consumer.
///   let client = async_nats::connect("demo.nats.io").await?;
///   let js = Arc::new(async_nats::jetstream::new(client));
///
///   let options = Arc::new(
///     AckSubOptions::new(Format::JSON, Arc::from("events"))
///       .subjects(vec!["events.user_created"])
///       .durable_name("user-created")
///       .auto_ack(false),
///   );
///
///   // SubFetcher implements both SubCtxTrait and UnSubTrait.
///   let fetcher = Arc::new(SubFetcher::new(js, options.clone()).await?);
///   let unsub = Some(fetcher.clone() as Arc<dyn UnSubTrait + Send + Sync>);
///
///   let subscriber: Sub<Event> = Sub::new(fetcher, unsub, options);
///   let mut stream = subscriber.subscribe().await?;
///
///   while let Some(Ok((event, ack))) = stream.next().await {
///     println!("received {:?}", event);
///     // Manually ack since auto_ack(false).
///     ack.ack().await?;
///   }
///
///   Ok(())
/// }
/// ```
pub struct Sub<T> {
  ctx: Arc<dyn SubCtxTrait + Send + Sync>,
  unsub: Option<Arc<dyn UnSubTrait + Send + Sync>>,
  options: Arc<dyn SubOptTrait + Send + Sync>,
  _marker: PhantomData<T>,
}

impl<T> Sub<T>
where
  T: DeserializeOwned + Send + Sync,
{
  /// Creates a new subscriber using the provided context, optional
  /// unsubscribe handler, and subscription options.
  ///
  /// # Parameters
  /// - `ctx`: Message retrieval context responsible for producing raw items.
  /// - `unsub`: Optional handler to cancel the subscription when requested.
  /// - `options`: Subscription behavior such as auto-ack and payload format.
  pub fn new(
    ctx: Arc<dyn SubCtxTrait + Send + Sync>,
    unsub: Option<Arc<dyn UnSubTrait + Send + Sync>>,
    options: Arc<dyn SubOptTrait + Send + Sync>,
  ) -> Self {
    Self {
      ctx,
      unsub,
      options,
      _marker: PhantomData,
    }
  }
}

#[async_trait]
impl<T> SubTrait for Sub<T>
where
  T: DeserializeOwned + Send + Sync,
{
  type Item = T;
  /// Returns a stream of decoded messages alongside their acknowledgment
  /// handles. When auto-acknowledgment is enabled, messages are acknowledged
  /// before being yielded to the consumer.
  async fn subscribe(
    &self,
  ) -> Result<
    BoxStream<Result<(Self::Item, Arc<dyn AckTrait + Send + Sync>), SubError>>,
    SubError,
  > {
    let messages = self.ctx.subscribe().await?;
    let stream = messages.and_then(async move |(msg, acker)| {
      if self.options.get_auto_ack() {
        acker.ack().await?;
      }
      let data = match self.options.get_format() {
        Format::MessagePack => {
          rmp_serde::from_slice::<T>(&msg).map_err(SubError::MessagePackDecode)
        }
        Format::JSON => {
          serde_json::from_slice::<T>(&msg).map_err(SubError::Json)
        }
      }?;
      Ok((data, acker))
    });
    return Ok(Box::pin(stream));
  }
}

#[async_trait]
impl<T> UnSubTrait for Sub<T>
where
  T: DeserializeOwned + Send + Sync,
{
  /// Invokes the optional unsubscribe handler, if present.
  async fn unsubscribe(&self) -> Result<(), UnSubError> {
    if let Some(unsub) = &self.unsub {
      unsub.unsubscribe().await?;
    }
    return Ok(());
  }
}

#[cfg(test)]
mod test {
  use ::bytes::Bytes;
  use ::futures::stream::StreamExt;
  use ::rmp_serde::to_vec as to_msgpack;
  use ::serde_json::to_vec as jsonify;

  use crate::errors::AckError;
  use crate::tests::{entity::TestEntity, subscribe::SubscribeMock};
  use crate::traits::{MockAckTrait, MockSubOptTrait};

  use super::*;

  async fn test_subscribe(format: Format, auto_ack: bool) {
    let entities = vec![
      TestEntity::new(1, "Test1"),
      TestEntity::new(2, "Test2"),
      TestEntity::new(3, "Test3"),
    ];
    let data: Vec<(Bytes, Arc<dyn AckTrait + Send + Sync>)> = entities
      .iter()
      .map(|e| {
        let mut ack_mock = MockAckTrait::new();
        if auto_ack {
          ack_mock.expect_ack().returning(|| Ok(())).once();
        } else {
          ack_mock.expect_ack().never();
        }
        return (
          Bytes::from(match format {
            Format::MessagePack => to_msgpack(e).unwrap(),
            Format::JSON => jsonify(e).unwrap(),
          }),
          Arc::new(ack_mock) as Arc<dyn AckTrait + Send + Sync>,
        );
      })
      .collect();
    let ctx: Arc<dyn SubCtxTrait + Send + Sync> =
      Arc::new(SubscribeMock::new(data));
    let mut options = MockSubOptTrait::new();
    options
      .expect_get_auto_ack()
      .return_const(auto_ack)
      .times(entities.len());
    options
      .expect_get_format()
      .return_const(format)
      .times(entities.len());
    let subscribe: Sub<TestEntity> = Sub::new(
      ctx,
      None,
      Arc::new(options) as Arc<dyn SubOptTrait + Send + Sync>,
    );
    let stream = subscribe.subscribe().await.unwrap();
    let obtained: Vec<TestEntity> = stream
      .try_collect::<Vec<_>>()
      .await
      .unwrap()
      .into_iter()
      .map(|(entity, _ack)| entity)
      .collect();
    assert_eq!(obtained, entities);
  }

  #[tokio::test]
  async fn test_subscribe_json() {
    test_subscribe(Format::JSON, true).await;
  }

  #[tokio::test]
  async fn test_subscribe_messagepack() {
    test_subscribe(Format::MessagePack, true).await;
  }

  #[tokio::test]
  async fn test_subscribe_json_no_auto_ack() {
    test_subscribe(Format::JSON, false).await;
  }

  #[tokio::test]
  async fn test_subscribe_messagepack_no_auto_ack() {
    test_subscribe(Format::MessagePack, false).await;
  }

  async fn test_ack_err(format: Format) {
    let mut data: Vec<(Bytes, Arc<dyn AckTrait + Send + Sync>)> = Vec::new();
    data.push((Bytes::new(), {
      let mut ack_mock = MockAckTrait::new();
      ack_mock
        .expect_ack()
        .returning(|| Err(AckError::ErrorTest))
        .once();
      Arc::new(ack_mock)
    }));
    let ctx: Arc<dyn SubCtxTrait + Send + Sync> =
      Arc::new(SubscribeMock::new(data));
    let mut options = MockSubOptTrait::new();
    options.expect_get_auto_ack().return_const(true).once();
    options.expect_get_format().return_const(format).never();
    let subscribe: Sub<TestEntity> = Sub::new(
      ctx,
      None,
      Arc::new(options) as Arc<dyn SubOptTrait + Send + Sync>,
    );
    let stream = subscribe.subscribe().await.unwrap();
    let obtained: Vec<String> = stream
      .collect::<Vec<_>>()
      .await
      .iter()
      .filter_map(|res| res.as_ref().map_err(|err| err.to_string()).err())
      .collect();
    assert_eq!(
      obtained,
      vec![SubError::AckError(AckError::ErrorTest).to_string()]
    );
  }

  #[tokio::test]
  async fn test_ack_json_err() {
    test_ack_err(Format::JSON).await;
  }

  #[tokio::test]
  async fn test_ack_messagepack_err() {
    test_ack_err(Format::MessagePack).await;
  }
}

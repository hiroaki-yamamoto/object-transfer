use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::{TryFutureExt, TryStreamExt};
use serde::de::{DeserializeOwned, Error as DeErr};

use crate::encoder::Decoder;
use crate::errors::{DecodeError, SubError, UnSubError};
use crate::options::SubOpt;
use crate::traits::{AckTrait, SubCtxTrait, SubTrait, UnSubTrait};

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
/// use object_transfer::nats::{SubFetcherOpt, SubFetcher};
/// use object_transfer::traits::{SubTrait, UnSubTrait};
/// use object_transfer::SubOpt;
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
///   let fetcher_option = SubFetcherOpt::new(Arc::from("events"))
///       .subjects(vec!["events.user_created"])
///       .durable_name("user-created");
///   let sub_option = SubOpt::new().format(Format::JSON);
///
///   // SubFetcher implements both SubCtxTrait and UnSubTrait.
///   let fetcher = Arc::new(SubFetcher::new(js, fetcher_option).await?);
///   let unsub = fetcher.clone();
///
///   let subscriber: Sub<Event> = Sub::new(fetcher, unsub, sub_option);
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
pub struct Sub<T, DecodeErrorType: DeErr + Send + Sync> {
  ctx: Arc<dyn SubCtxTrait + Send + Sync>,
  unsub: Arc<dyn UnSubTrait + Send + Sync>,
  decoder: Arc<dyn Decoder<Item = T, Error = DecodeErrorType> + Send + Sync>,
  options: SubOpt,
  _marker: PhantomData<T>,
}

impl<T, DecodeErrorType> Sub<T, DecodeErrorType>
where
  T: DeserializeOwned + Send + Sync,
  DecodeErrorType: DeErr + Send + Sync,
{
  /// Creates a new subscriber using the provided context, optional
  /// unsubscribe handler, and subscription options.
  ///
  /// # Parameters
  /// - `ctx`: Message retrieval context responsible for producing raw items.
  /// - `unsub`: Unsubscribe handler to cancel the subscription when requested.
  /// - `options`: Subscription behavior such as auto-ack and payload format.
  pub fn new(
    ctx: Arc<dyn SubCtxTrait + Send + Sync>,
    unsub: Arc<dyn UnSubTrait + Send + Sync>,
    decoder: Arc<dyn Decoder<Item = T, Error = DecodeErrorType> + Send + Sync>,
    options: SubOpt,
  ) -> Self {
    Self {
      ctx,
      unsub,
      decoder,
      options,
      _marker: PhantomData,
    }
  }
}

#[async_trait]
impl<T, DecodeErrorType> SubTrait for Sub<T, DecodeErrorType>
where
  T: DeserializeOwned + Send + Sync,
  DecodeErrorType: DeErr + Send + Sync,
{
  type Item = T;
  type DecodeErr = DecodeErrorType;
  /// Returns a stream of decoded messages alongside their acknowledgment
  /// handles. When auto-acknowledgment is enabled, messages are acknowledged
  /// before being yielded to the consumer.
  async fn subscribe(
    &self,
  ) -> Result<
    BoxStream<
      Result<
        (Self::Item, Arc<dyn AckTrait + Send + Sync>),
        SubError<Self::DecodeErr>,
      >,
    >,
    SubError<Self::DecodeErr>,
  > {
    let messages = self.ctx.subscribe().await?.map_err(SubError::from);
    let stream = messages.and_then(async move |(msg, acker)| {
      if self.options.auto_ack {
        acker.ack().map_err(|e| SubError::AckError(e)).await?;
      }
      let data = self
        .decoder
        .decode(msg)
        .map_err(|e| SubError::from(DecodeError::new(e)))?;
      Ok((data, acker))
    });
    Ok(Box::pin(stream))
  }
}

#[async_trait]
impl<T, DecodeErrorType> UnSubTrait for Sub<T, DecodeErrorType>
where
  T: DeserializeOwned + Send + Sync,
  DecodeErrorType: DeErr + Send + Sync,
{
  /// Invokes the configured unsubscribe handler.
  async fn unsubscribe(&self) -> Result<(), UnSubError> {
    self.unsub.unsubscribe().await
  }
}

#[cfg(test)]
mod test {
  use ::bytes::Bytes;
  use ::futures::stream::StreamExt;
  use ::mockall::predicate::*;
  use ::serde_json::{from_slice as parse, to_vec as jsonify};

  use crate::UnSubNoop;
  use crate::encoder::MockDecoder;
  use crate::errors::AckError;
  use crate::tests::{
    entity::TestEntity, error::MockDeErr, subscribe::SubscribeMock,
  };
  use crate::traits::MockAckTrait;

  use super::*;

  async fn test_subscribe(auto_ack: bool) {
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
          Bytes::from(jsonify(e).unwrap()),
          Arc::new(ack_mock) as Arc<dyn AckTrait + Send + Sync>,
        );
      })
      .collect();
    let mut decoder = MockDecoder::new();
    let mut expect_decoder = decoder.expect_decode().returning(|bytes| {
      let entity: TestEntity = parse(&bytes).unwrap();
      Ok(entity)
    });
    for item in &data {
      expect_decoder = expect_decoder.with(eq(item.0.clone()));
    }
    let ctx: Arc<dyn SubCtxTrait + Send + Sync> =
      Arc::new(SubscribeMock::new(data));
    let options = SubOpt::new().auto_ack(auto_ack);
    let subscribe: Sub<TestEntity, _> = Sub::new(
      ctx,
      Arc::new(UnSubNoop::new(false)),
      Arc::new(decoder),
      options,
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
  async fn test_subscribe_ack() {
    test_subscribe(true).await;
  }

  #[tokio::test]
  async fn test_subscribe_no_auto_ack() {
    test_subscribe(false).await;
  }

  #[tokio::test]
  async fn test_ack_err() {
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
    let mut decoder = MockDecoder::new();
    decoder.expect_decode().never();
    let options = SubOpt::new().auto_ack(true);
    let subscribe: Sub<TestEntity, _> = Sub::new(
      ctx,
      Arc::new(UnSubNoop::new(false)),
      Arc::new(decoder),
      options,
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
      vec![SubError::<MockDeErr>::AckError(AckError::ErrorTest).to_string()]
    );
  }
}

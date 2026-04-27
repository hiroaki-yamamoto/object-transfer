use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::{TryFutureExt, TryStreamExt};
use serde::de::{DeserializeOwned, Error as DeErr};

use crate::brokers::SubBrokerTrait;
use crate::encoders::Decoder;
use crate::errors::{DecodeError, SubError, UnSubError};
use crate::options::SubOpt;
use crate::traits::{AckTrait, SubTrait, UnSubTrait};

/// Subscriber wrapper that deserializes messages and optionally acknowledges them.
///
/// The subscriber uses a pluggable [`Decoder`] to deserialize messages,
/// relies on a [`SubBrokerTrait`] implementation for message retrieval, and optionally
/// acknowledges them based on [`SubOpt`] settings. The decoder is passed at construction time,
/// enabling runtime format selection.
///
/// # Example with JSON
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use futures::StreamExt;
/// use serde::Deserialize;
/// use object_transfer::{
///   encoders::JSONDecoder,
///   Sub, SubOpt,
/// };
/// use object_transfer::brokers::nats::{SubFetcherOpt, SubFetcher};
/// use object_transfer::traits::{SubTrait};
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
///
///   // SubFetcher implements both SubCtxTrait and UnSubTrait.
///   let fetcher = Arc::new(SubFetcher::new(js, fetcher_option).await?);
///   let unsub = fetcher.clone();
///
///   let options = SubOpt::new().auto_ack(false);
///   let subscriber: Sub<Event, _> = Sub::new(
///     fetcher,
///     unsub,
///     Arc::new(JSONDecoder::new()),
///     options,
///   );
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
///
/// # Example with Custom Format
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use bytes::Bytes;
/// use serde::de::DeserializeOwned;
/// use object_transfer::{
///   encoders::Decoder,
///   Sub, SubOpt,
///   traits::SubTrait,
/// };
///
/// struct CustomDecoder;
///
/// #[derive(Debug, serde::Deserialize)]
/// struct MyType {
///   data: String,
/// }
///
/// #[derive(Debug)]
/// enum CustomError {
///   InvalidData,
/// }
///
/// impl std::fmt::Display for CustomError {
///   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///     write!(f, "Custom decode error")
///   }
/// }
///
/// impl std::error::Error for CustomError {}
///
/// impl serde::de::Error for CustomError {
///   fn custom<T: std::fmt::Display>(_msg: T) -> Self {
///     CustomError::InvalidData
///   }
/// }
///
/// impl Decoder for CustomDecoder {
///   type Item = MyType;
///   type Error = CustomError;
///
///   fn decode(&self, data: Bytes) -> Result<Self::Item, Self::Error> {
///     let text = String::from_utf8(data.to_vec())
///       .map_err(|_| CustomError::InvalidData)?;
///     Ok(MyType { data: text })
///   }
/// }
/// ```
pub struct Sub<T, DecodeErrorType: DeErr + Send + Sync> {
  ctx: Arc<dyn SubBrokerTrait + Send + Sync>,
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
  /// Creates a new subscriber using the provided context, decoder, and options.
  ///
  /// # Parameters
  /// - `ctx`: Message retrieval context responsible for producing raw bytes.
  /// - `unsub`: Unsubscribe handler to cancel the subscription when requested.
  /// - `decoder`: A trait object implementing [`Decoder`] for your format.
  ///   Pass `Arc::new(JSONDecoder::new())`, `Arc::new(MessagePackDecoder::new())`, or
  ///   your custom decoder implementation.
  /// - `options`: Subscription behavior such as auto-acknowledgment settings.
  ///
  /// # Decoder Selection
  ///
  /// The decoder is passed at construction time, allowing for:
  /// - **Compile-time format selection**: Create different subscriber instances with different types
  /// - **Runtime format selection**: Use `Arc<dyn Decoder<...>>` to select format dynamically
  ///
  /// # Error Types
  ///
  /// The generic `DecodeErrorType` type parameter is the error type of your decoder. Different decoders
  /// can use different error types (e.g., `serde_json::Error`, custom parse error types).
  pub fn new(
    ctx: Arc<dyn SubBrokerTrait + Send + Sync>,
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
      let data = self
        .decoder
        .decode(msg)
        .map_err(|e| SubError::from(DecodeError::new(e)))?;
      if self.options.auto_ack {
        acker.ack().map_err(|e| SubError::AckError(e)).await?;
      }
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
  use ::serde_json::{from_slice as parse, to_vec as jsonify};

  use crate::UnSubNoop;
  use crate::encoders::MockDecoder;
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
    decoder
      .expect_decode()
      .times(entities.len())
      .returning(|bytes| {
        let entity: TestEntity = parse(&bytes).unwrap();
        Ok(entity)
      });
    let ctx: Arc<dyn SubBrokerTrait + Send + Sync> =
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
    let ctx: Arc<dyn SubBrokerTrait + Send + Sync> =
      Arc::new(SubscribeMock::new(data));
    let mut decoder = MockDecoder::new();
    decoder.expect_decode().once().returning(|_| {
      let entity = TestEntity::new(0, "Test");
      Ok(entity)
    });
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

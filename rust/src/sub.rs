use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;
use futures::TryStreamExt;
use futures::stream::BoxStream;
use serde::de::DeserializeOwned;

use crate::r#enum::Format;
use crate::error::Error;
use crate::traits::{
  AckTrait, SubCtxTrait, SubOptTrait, SubTrait, UnSubTrait,
};

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
  pub async fn new(
    ctx: Arc<dyn SubCtxTrait + Send + Sync>,
    unsub: Option<Arc<dyn UnSubTrait + Send + Sync>>,
    options: Arc<dyn SubOptTrait + Send + Sync>,
  ) -> Result<Self, Error> {
    Ok(Self {
      ctx,
      unsub,
      options,
      _marker: PhantomData,
    })
  }
}

#[async_trait]
impl<T> SubTrait for Sub<T>
where
  T: DeserializeOwned + Send + Sync,
{
  type Item = T;
  async fn subscribe(
    &self,
  ) -> Result<
    BoxStream<Result<(Self::Item, Arc<dyn AckTrait + Send + Sync>), Error>>,
    Error,
  > {
    let messages = self.ctx.subscribe().await?;
    let stream = messages.and_then(async move |(msg, acker)| {
      if self.options.get_auto_ack() {
        acker.ack().await?;
      }
      let data = match self.options.get_format() {
        Format::MessagePack => {
          rmp_serde::from_slice::<T>(&msg).map_err(Error::MessagePackDecode)
        }
        Format::JSON => serde_json::from_slice::<T>(&msg).map_err(Error::Json),
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
  async fn unsubscribe(&self) -> Result<(), Error> {
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
    )
    .await
    .unwrap();
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
        .returning(|| Err(Error::ErrorTest))
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
    )
    .await
    .unwrap();
    let stream = subscribe.subscribe().await.unwrap();
    let obtained: Vec<String> = stream
      .collect::<Vec<_>>()
      .await
      .iter()
      .filter_map(|res| res.as_ref().map_err(|err| err.to_string()).err())
      .collect();
    assert_eq!(obtained, vec![Error::ErrorTest.to_string()]);
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

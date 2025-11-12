use ::bytes::Bytes;

use ::async_trait::async_trait;
use ::futures::stream::BoxStream;
use ::serde::{Serialize, de::DeserializeOwned};

use crate::r#enum::Format;
use crate::error::Error;

#[cfg(test)]
use crate::tests::entity::TestEntity;
#[cfg(test)]
use ::mockall::automock;

#[cfg_attr(test, automock(type Item = TestEntity;))]
#[async_trait]
pub trait PubTrait {
  type Item: Serialize + Send + Sync;
  async fn publish(&self, obj: &Self::Item) -> Result<(), Error>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait AckTrait {
  async fn ack(&self) -> Result<(), Error>;
}

#[async_trait]
pub trait SubTrait {
  type Item: DeserializeOwned + Send + Sync;
  async fn subscribe(
    &self,
  ) -> Result<
    BoxStream<Result<(Self::Item, Box<dyn AckTrait + Send>), Error>>,
    Error,
  >;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait UnSubTrait {
  async fn unsubscribe(&self) -> Result<(), Error>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait PubCtxTrait {
  async fn publish(&self, topic: &str, payload: Bytes) -> Result<(), Error>;
}

#[async_trait]
pub trait SubCtxTrait {
  async fn subscribe(
    &self,
  ) -> Result<BoxStream<Result<(Bytes, Box<dyn AckTrait + Send>), Error>>, Error>;
}

#[cfg_attr(test, automock)]
pub trait SubOptTrait {
  fn get_auto_ack(&self) -> bool;
  fn get_format(&self) -> Format;
}

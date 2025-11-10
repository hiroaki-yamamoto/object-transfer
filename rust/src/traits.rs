use ::bytes::Bytes;

use ::futures::stream::BoxStream;
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};

use crate::r#enum::Format;
use crate::error::Error;

#[async_trait]
pub trait PubTrait {
  type Item: Serialize + Send + Sync;
  async fn publish(&self, obj: &Self::Item) -> Result<(), Error>;
}

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

#[async_trait]
pub trait UnSubTrait {
  async fn unsubscribe(&self) -> Result<(), Error>;
}

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

pub trait SubOptTrait {
  fn get_auto_ack(&self) -> bool;
  fn get_format(&self) -> Format;
}

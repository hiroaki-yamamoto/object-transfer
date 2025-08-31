use ::bytes::Bytes;
use ::futures::Stream;

use crate::error::Error;
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};

#[async_trait]
pub trait PubTrait {
  async fn publish<T>(&self, obj: &T) -> Result<(), Error>
  where
    T: Serialize + Send + Sync;
}

#[async_trait]
pub trait AckTrait {
  async fn ack(&self) -> Result<(), Error>;
}

#[async_trait]
pub trait SubTrait<T>
where
  T: DeserializeOwned + Send + Sync,
{
  async fn subscribe(
    &self,
  ) -> Result<
    impl Stream<Item = Result<(T, impl AckTrait), Error>> + Send + Sync,
    Error,
  >;
}

#[async_trait]
pub trait UnSubTrait {
  async fn unsubscribe(&self) -> Result<(), Error>;
}

#[async_trait]
pub trait PubContext {
  async fn publish(&self, topic: &str, payload: Bytes) -> Result<(), Error>;
}

#[async_trait]
pub trait SubContext {
  async fn subscribe(
    &self,
    topic: &str,
  ) -> Result<impl Stream<Item = Result<Bytes, Error>> + Send + Sync, Error>;
}

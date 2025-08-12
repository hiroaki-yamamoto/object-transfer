use crate::error::Error;
use async_trait::async_trait;
use futures::Stream;
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

use crate::error::Error;
use async_trait::async_trait;
use futures::Stream;
use serde::Serialize;

#[async_trait]
pub trait PubTrait {
  async fn publish<T>(&self, obj: &T) -> Result<(), Error>
  where
    T: Serialize + Send + Sync;
}

pub trait SubTrait<T>: Stream<Item = Result<T, Error>> + Send {}

impl<T, S> SubTrait<T> for S where S: Stream<Item = Result<T, Error>> + Send {}

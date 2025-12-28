use ::std::sync::Arc;

use ::async_trait::async_trait;
use ::bytes::Bytes;
use ::futures::stream::{BoxStream, StreamExt, iter};
use ::serde::de::DeserializeOwned;

use crate::errors::SubError;
use crate::traits::{AckTrait, SubCtxTrait, SubTrait};

pub struct SubscribeMock<Entity> {
  data: Vec<(Entity, Arc<dyn AckTrait + Send + Sync>)>,
}

impl<Entity> SubscribeMock<Entity> {
  pub fn new(data: Vec<(Entity, Arc<dyn AckTrait + Send + Sync>)>) -> Self {
    Self { data: data }
  }
}

#[async_trait]
impl<Entity> SubTrait for SubscribeMock<Entity>
where
  Entity: DeserializeOwned + Clone + Send + Sync,
{
  type Item = Entity;

  async fn subscribe(
    &self,
  ) -> Result<
    BoxStream<Result<(Self::Item, Arc<dyn AckTrait + Send + Sync>), SubError>>,
    SubError,
  > {
    return Ok(iter(self.data.clone()).map(|item| Ok(item)).boxed());
  }
}

#[async_trait]
impl SubCtxTrait for SubscribeMock<Bytes> {
  async fn subscribe(
    &self,
  ) -> Result<
    BoxStream<Result<(Bytes, Arc<dyn AckTrait + Send + Sync>), SubError>>,
    SubError,
  > {
    return Ok(iter(self.data.clone()).map(|item| Ok(item)).boxed());
  }
}

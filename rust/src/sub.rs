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
    BoxStream<Result<(Self::Item, Box<dyn AckTrait + Send>), Error>>,
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
      Ok((data, acker as Box<dyn AckTrait + Send>))
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

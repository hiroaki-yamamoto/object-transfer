use ::std::sync::Arc;

use ::async_nats::jetstream::{context::Context, stream::Stream as JStream};
use ::async_trait::async_trait;
use ::bytes::Bytes;
use ::futures::StreamExt;
use ::futures::stream::BoxStream;

use super::options::AckSubOptions;
use crate::error::Error;
use crate::traits::{AckTrait, SubCtxTrait, UnSubTrait};

#[derive(Debug)]
pub struct SubFetcher {
  ctx: Context,
  options: Arc<AckSubOptions>,
}

impl SubFetcher {
  pub fn new(ctx: Context, options: Arc<AckSubOptions>) -> Self {
    return Self { ctx, options };
  }
  async fn get_stream(&self) -> Result<JStream, Error> {
    return Ok(
      self
        .ctx
        .get_or_create_stream(self.options.stream_cfg.clone())
        .await?,
    );
  }
}

#[async_trait]
impl SubCtxTrait for SubFetcher {
  async fn subscribe(
    &self,
  ) -> Result<
    BoxStream<'async_trait, Result<(Bytes, Box<dyn AckTrait + Send>), Error>>,
    Error,
  > {
    let stream = self.get_stream().await?;
    let consumer = stream
      .get_or_create_consumer(
        &self.options.stream_cfg.name,
        self.options.pull_cfg.clone(),
      )
      .await?;
    let messages = async_stream::try_stream! {
      let mut msgs = consumer.subscribe().await?;
      while let Some(result) = msgs.next().await {
        yield result?;
      }
    };
    Ok(messages.boxed())
  }
}

#[async_trait]
impl UnSubTrait for SubFetcher {
  async fn unsubscribe(&self) -> Result<(), Error> {
    let stream = self.get_stream().await?;
    stream
      .delete_consumer(&self.options.pull_cfg.durable_name.clone().unwrap())
      .await?;
    Ok(())
  }
}

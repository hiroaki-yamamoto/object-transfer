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
  stream: JStream,
  options: Arc<AckSubOptions>,
}

impl SubFetcher {
  pub async fn new(
    ctx: Arc<Context>,
    options: Arc<AckSubOptions>,
  ) -> Result<Self, Error> {
    let stream = ctx.get_or_create_stream(options.stream_cfg.clone()).await?;
    Ok(Self { stream, options })
  }
}

#[async_trait]
impl SubCtxTrait for SubFetcher {
  async fn subscribe(
    &self,
  ) -> Result<
    BoxStream<Result<(Bytes, Arc<dyn AckTrait + Send + Sync>), Error>>,
    Error,
  > {
    let consumer = self
      .stream
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
    self
      .stream
      .delete_consumer(&self.options.pull_cfg.durable_name.clone().unwrap())
      .await?;
    Ok(())
  }
}

use ::std::sync::Arc;

use ::async_nats::jetstream::{context::Context, stream::Stream as JStream};
use ::async_trait::async_trait;
use ::bytes::Bytes;
use ::futures::{Stream, TryFutureExt, TryStreamExt};

use super::options::AckSubOptions;
use crate::error::Error;
use crate::traits::SubCtxTrait;

#[derive(Debug)]
pub(super) struct SubFetcher {
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
  ) -> Result<impl Stream<Item = Result<Bytes, Error>> + Send + Sync, Error>
  {
    let stream = self.get_stream().await?;
    let consumer = stream
      .get_or_create_consumer(
        &self.options.stream_cfg.name,
        self.options.pull_cfg.clone(),
      )
      .await?;
    let messages = consumer
      .messages()
      .map_ok(|stream| {
        return stream
          .map_ok(|msg| msg.payload.clone())
          .map_err(Error::from);
      })
      .await?;
    Ok(messages)
  }
}

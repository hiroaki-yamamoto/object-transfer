use ::std::sync::Arc;

use ::async_nats::jetstream::{context::Context, stream::Stream as JStream};
use ::async_trait::async_trait;
use ::bytes::Bytes;
use ::futures::StreamExt;
use ::futures::stream::BoxStream;

use super::errors::NatsSubFetcherError;
use super::options::AckSubOptions;
use crate::errors::{SubError, UnSubError};
use crate::traits::{AckTrait, SubCtxTrait, UnSubTrait};

/// Fetches pull-based JetStream messages using the configured stream options.
#[derive(Debug)]
pub struct SubFetcher {
  stream: JStream,
  options: Arc<AckSubOptions>,
}

impl SubFetcher {
  /// Creates or reuses a JetStream stream based on the provided options.
  ///
  /// # Parameters
  /// - `ctx`: JetStream context used to resolve or create the target stream.
  /// - `options`: Configuration for the stream and durable pull consumer.
  pub async fn new(
    ctx: Arc<Context>,
    options: Arc<AckSubOptions>,
  ) -> Result<Self, NatsSubFetcherError> {
    let stream = ctx.get_or_create_stream(options.stream_cfg.clone()).await?;
    Ok(Self { stream, options })
  }
}

#[async_trait]
impl SubCtxTrait for SubFetcher {
  /// Stream messages from the pull consumer, yielding their payloads along
  /// with the associated acknowledgment handles.
  async fn subscribe(
    &self,
  ) -> Result<
    BoxStream<Result<(Bytes, Arc<dyn AckTrait + Send + Sync>), SubError>>,
    SubError,
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
  /// Deletes the durable consumer associated with this fetcher.
  async fn unsubscribe(&self) -> Result<(), UnSubError> {
    self
      .stream
      .delete_consumer(&self.options.pull_cfg.durable_name.clone().unwrap())
      .await?;
    Ok(())
  }
}

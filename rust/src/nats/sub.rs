use std::marker::PhantomData;
use std::sync::Arc;

use async_nats::jetstream::{self, stream::Stream as JStream};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde::de::DeserializeOwned;

use crate::r#enum::Format;
use crate::error::Error;
use crate::traits::{AckTrait, SubTrait, UnSubTrait};

use super::options::AckSubOptions;

#[derive(Debug)]
pub struct Sub<T> {
  stream: JStream,
  options: Arc<AckSubOptions>,
  _marker: PhantomData<T>,
}

impl<T> Sub<T>
where
  T: DeserializeOwned,
{
  pub async fn new(
    js: jetstream::Context,
    options: AckSubOptions,
  ) -> Result<Self, Error> {
    let stream = js.get_or_create_stream(options.stream_cfg.clone()).await?;
    Ok(Self {
      stream,
      options: Arc::new(options),
      _marker: PhantomData,
    })
  }
}

#[async_trait]
impl<T> SubTrait<T> for Sub<T>
where
  T: DeserializeOwned + Send + Sync,
{
  async fn subscribe(
    &self,
  ) -> Result<
    impl Stream<Item = Result<(T, impl AckTrait), Error>> + Send + Sync,
    Error,
  > {
    let options = self.options.clone();
    let consumer = self
      .stream
      .get_or_create_consumer(
        &self.options.stream_cfg.name,
        self.options.pull_cfg.clone(),
      )
      .await?;

    let messages = consumer.messages().await?;
    let stream = messages.then(move |msg_res| {
      let format = options.format;
      let options = options.clone();
      async move {
        let msg = msg_res?;
        let (msg, acker) = msg.split();
        if options.auto_ack {
          acker.ack().await?;
        }
        let data = match format {
          Format::MessagePack => rmp_serde::from_slice::<T>(&msg.payload)
            .map_err(Error::MessagePackDecode),
          Format::JSON => {
            serde_json::from_slice::<T>(&msg.payload).map_err(Error::Json)
          }
        }?;
        Ok((data, acker))
      }
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
    self
      .stream
      .delete_consumer(&self.options.stream_cfg.name)
      .await?;
    Ok(())
  }
}

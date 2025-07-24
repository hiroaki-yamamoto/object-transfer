use std::pin::Pin;
use std::task::{Context, Poll};

use async_nats::jetstream;
use futures::stream::BoxStream;
use futures::{Stream, StreamExt};
use serde::de::DeserializeOwned;

use crate::r#enum::Format;
use crate::error::Error;

use super::options::AckSubOptions;

pub struct Sub<T> {
  stream: BoxStream<'static, Result<T, Error>>,
}

impl<T> Sub<T>
where
  T: DeserializeOwned + Send + 'static,
{
  pub async fn new(
    js: jetstream::Context,
    options: AckSubOptions,
  ) -> Result<Self, Error> {
    let name = options.stream_cfg.name.clone();
    let stream = js.get_or_create_stream(options.stream_cfg.clone()).await?;
    let consumer = stream
      .get_or_create_consumer(&name, options.pull_cfg.clone())
      .await?;

    let messages = consumer.messages().await?;
    let stream = messages.then(move |msg_res| {
      let format = options.format;
      async move {
        let msg = msg_res?;
        if options.auto_ack {
          msg.ack().await?;
        }
        let data = match format {
          Format::MessagePack => {
            rmp_serde::from_slice::<T>(&msg.message.payload)
              .map_err(Error::MessagePackDecode)
          }
          Format::JSON => serde_json::from_slice::<T>(&msg.message.payload)
            .map_err(Error::Json),
        }?;
        Ok(data)
      }
    });
    Ok(Self {
      stream: Box::pin(stream),
    })
  }
}

impl<T> Stream for Sub<T>
where
  T: DeserializeOwned + Send + 'static,
{
  type Item = Result<T, Error>;

  fn poll_next(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    let inner = self.get_mut();
    Pin::new(&mut inner.stream).poll_next(cx)
  }
}

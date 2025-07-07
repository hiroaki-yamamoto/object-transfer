use crate::r#enum::Format;
use crate::error::Error;
use async_nats::jetstream::{
  self, consumer::pull::Config as PullConfig, stream::Config as StreamConfig,
};
use futures::stream::BoxStream;
use futures::{Stream, StreamExt};
use serde::de::DeserializeOwned;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct Sub<T> {
  stream: BoxStream<'static, Result<T, Error>>,
}

impl<T> Sub<T>
where
  T: DeserializeOwned + Send + 'static,
{
  pub async fn new(
    js: jetstream::Context,
    name: impl Into<String> + Clone,
    subjects: Vec<impl Into<String>>,
    format: Format,
    durable_name: Option<impl Into<String>>,
  ) -> Result<Self, Error> {
    let stream = js
      .get_or_create_stream(StreamConfig {
        name: name.clone().into(),
        subjects: subjects.into_iter().map(Into::into).collect(),
        ..Default::default()
      })
      .await?;
    let consumer = stream
      .get_or_create_consumer(
        name.into().as_str(),
        PullConfig {
          durable_name: durable_name.map(Into::into),
          ..Default::default()
        },
      )
      .await?;

    let messages = consumer.messages().await?;
    let stream = messages.then(move |msg_res| {
      let format = format;
      async move {
        let msg = msg_res.map_err(|e| Error::Other(e.to_string()))?;
        let data = match format {
          Format::MessagePack => {
            rmp_serde::from_slice::<T>(&msg.message.payload)
              .map_err(Error::MessagePackDecode)
          }
          Format::JSON => serde_json::from_slice::<T>(&msg.message.payload)
            .map_err(Error::Json),
        }?;
        msg.ack().await?;
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

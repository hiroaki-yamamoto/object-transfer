use ::async_nats::jetstream::Context;
use ::async_nats::jetstream::consumer::{
  PullConsumer as JSPullCons, PushConsumer as JSPushCons,
};
use ::async_trait::async_trait;
use ::bytes::Bytes;
use ::futures::{Stream, TryFutureExt, TryStreamExt};

use crate::error::Error;
use crate::traits::{AckTrait, PubCtxTrait, SubCtxTrait};

#[async_trait]
impl PubCtxTrait for Context {
  async fn publish(&self, topic: &str, payload: Bytes) -> Result<(), Error> {
    self.publish(topic.to_string(), payload).await?.await?;
    return Ok(());
  }
}

macro_rules! impl_js_consumer {
  ($name:ident) => {
    #[async_trait]
    impl SubCtxTrait for $name {
      async fn subscribe(
        &self,
      ) -> Result<
        impl Stream<Item = Result<(Bytes, impl AckTrait), Error>>
        + Send
        + Sync,
        Error,
      > {
        let messages = self
          .messages()
          .map_ok(|stream| {
            return stream.map_err(Error::from).and_then(async |msg| {
              let (msg, acker) = msg.split();
              return Ok((msg.payload.clone(), acker));
            });
          })
          .await?;
        Ok(messages)
      }
    }
  };
}

impl_js_consumer!(JSPullCons);
impl_js_consumer!(JSPushCons);
// TODO: Need idea to implement SubCtxTrait for OrderedConsumer

use ::async_nats::jetstream::Context;
use ::async_nats::jetstream::consumer::{
  pull::Stream as PullMsgs, push::Messages as PushMsgs,
};
use ::async_trait::async_trait;
use ::bytes::Bytes;
use ::futures::stream::BoxStream;
use ::futures::{StreamExt, TryStreamExt};
use ::std::boxed::Box;

use crate::error::Error;
use crate::traits::{AckTrait, PubCtxTrait, SubCtxTrait};

#[async_trait]
impl PubCtxTrait for Context {
  async fn publish(&self, topic: &str, payload: Bytes) -> Result<(), Error> {
    self.publish(topic.to_string(), payload).await?.await?;
    return Ok(());
  }
}

macro_rules! impl_sub_ctx_trait {
  ($cls_name: ty) => {
    #[async_trait]
    impl SubCtxTrait for $cls_name {
      async fn subscribe(
        self,
      ) -> Result<
        BoxStream<
          'async_trait,
          Result<(Bytes, Box<dyn AckTrait + Send>), Error>,
        >,
        Error,
      > {
        let messages = self.map_err(Error::from).and_then(async |msg| {
          let (msg, acker) = msg.split();
          return Ok((
            msg.payload.clone(),
            Box::new(acker) as Box<dyn AckTrait + Send>,
          ));
        });
        Ok(messages.boxed())
      }
    }
  };
}

impl_sub_ctx_trait!(PullMsgs);
impl_sub_ctx_trait!(PushMsgs);

// TODO: Need idea to implement SubCtxTrait for Ordered Message

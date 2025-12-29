use ::std::sync::Arc;

use ::async_nats::jetstream::Context;
use ::async_nats::jetstream::consumer::{
  PullConsumer as PullCons, PushConsumer as PushCons,
};
use ::async_trait::async_trait;
use ::bytes::Bytes;
use ::futures::stream::BoxStream;
use ::futures::{StreamExt, TryFutureExt, TryStreamExt};
use ::std::boxed::Box;

use crate::errors::{PubError, SubError};
use crate::traits::{AckTrait, PubCtxTrait, SubCtxTrait};

#[async_trait]
impl PubCtxTrait for Context {
  async fn publish(
    &self,
    topic: &str,
    payload: Bytes,
  ) -> Result<(), PubError> {
    self
      .publish(topic.to_string(), payload)
      .map_err(|e| PubError::BrokerError(e.into()))
      .await?
      .await
      .map_err(|e| PubError::BrokerError(e.into()))?;
    Ok(())
  }
}

macro_rules! impl_sub_ctx_trait {
  ($cls_name: ty) => {
    #[async_trait]
    impl SubCtxTrait for $cls_name {
      async fn subscribe(
        &self,
      ) -> Result<
        BoxStream<Result<(Bytes, Arc<dyn AckTrait + Send + Sync>), SubError>>,
        SubError,
      > {
        let messages = self
          .messages()
          .map_err(|e| SubError::BrokerError(e.into()))
          .await?
          .map_err(|e| SubError::BrokerError(e.into()))
          .and_then(async |msg| {
            let (msg, acker) = msg.split();
            Ok((
              msg.payload.clone(),
              Arc::new(acker) as Arc<dyn AckTrait + Send + Sync>,
            ))
          });
        Ok(messages.boxed())
      }
    }
  };
}

impl_sub_ctx_trait!(PullCons);
impl_sub_ctx_trait!(PushCons);

// TODO: Need idea to implement SubCtxTrait for Ordered Message

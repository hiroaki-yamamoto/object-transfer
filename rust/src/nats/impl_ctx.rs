use ::async_nats::jetstream::Context;
// use ::async_nats::jetstream::stream::Stream as NatsStream;
use ::async_trait::async_trait;
use ::bytes::Bytes;
// use ::futures::Stream;

use crate::error::Error;
use crate::traits::{
  PubCtxTrait,
  // SubCtxTrait,
};

#[async_trait]
impl PubCtxTrait for Context {
  async fn publish(&self, topic: &str, payload: Bytes) -> Result<(), Error> {
    self.publish(topic.to_string(), payload).await?.await?;
    return Ok(());
  }
}

// #[async_trait]
// impl SubCtxTrait for NatsStream {
//   async fn subscribe(
//     &self,
//     topic: &str,
//   ) -> Result<impl Stream<Item = Result<Bytes, Error>> + Send + Sync, Error>
//   {
//     let consumer = self
//       .get_or_create_consumer(
//         &self.options.stream_cfg.name,
//         self.options.pull_cfg.clone(),
//       )
//       .await?;

//     let messages = consumer.messages().await?;
//     return Ok(messages);
//   }
// }

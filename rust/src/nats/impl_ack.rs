use async_nats::jetstream::Message;
use async_trait::async_trait;
use futures::TryFutureExt;

use crate::traits::AckTrait;

#[async_trait]
impl AckTrait for Message {
  async fn ack(&self) -> Result<(), crate::error::Error> {
    return self.ack().map_err(|e| e.into()).await;
  }
}

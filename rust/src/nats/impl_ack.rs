use async_nats::jetstream::message::Acker;
use async_trait::async_trait;
use futures::TryFutureExt;

use crate::error::AckError;
use crate::traits::AckTrait;

#[async_trait]
impl AckTrait for Acker {
  async fn ack(&self) -> Result<(), AckError> {
    return self.ack().map_err(|e| e.into()).await;
  }
}

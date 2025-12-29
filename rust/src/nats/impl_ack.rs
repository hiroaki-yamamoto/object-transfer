use async_nats::jetstream::message::Acker;
use async_trait::async_trait;
use futures::TryFutureExt;

use crate::errors::AckError;
use crate::traits::AckTrait;

#[async_trait]
impl AckTrait for Acker {
  async fn ack(&self) -> Result<(), AckError> {
    self
      .ack()
      .map_err(|e| AckError::BrokerError(e.into()))
      .await
  }
}

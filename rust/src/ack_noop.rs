use crate::traits::AckTrait;
use ::async_trait::async_trait;

pub struct AckNoop;

#[async_trait]
impl AckTrait for AckNoop {
  async fn ack(&self) -> Result<(), crate::error::Error> {
    Ok(())
  }
}

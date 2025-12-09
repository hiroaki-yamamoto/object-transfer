use crate::traits::AckTrait;
use ::async_trait::async_trait;

/// Acknowledgment handler that performs no operation.
///
/// This can be used in scenarios where an acknowledgment callback is
/// required by the interface but no actual acknowledgement should be sent
/// to the message broker.
pub struct AckNoop;

#[async_trait]
impl AckTrait for AckNoop {
  async fn ack(&self) -> Result<(), crate::error::Error> {
    Ok(())
  }
}

use ::std::marker::PhantomData;
use ::std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;
use serde::ser::Error as EncErr;

use crate::encoder::Encoder;
use crate::errors::{EncodeError, PubError};
use crate::traits::{PubCtxTrait, PubTrait};

/// Publisher for serializable messages using a pluggable context.
///
/// The publisher encodes messages according to the configured [`Format`] and
/// delegates the actual publish call to an injected [`PubCtxTrait`] so it can
/// work with different backends.
///
/// # Example
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use serde::Serialize;
/// use object_transfer::{Format, Pub};
/// use object_transfer::traits::{PubCtxTrait, PubTrait};
///
/// #[derive(Serialize)]
/// struct UserCreated {
///   id: u64,
///   name: String,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///   let client = async_nats::connect("demo.nats.io").await?;
///   let js = async_nats::jetstream::new(client);
///
///   // JetStream context satisfies `PubCtxTrait`, so we can publish typed events.
///   let publisher: Pub<UserCreated> = Pub::new(
///     Arc::new(js),
///     "events.user_created",
///     Format::JSON,
///   );
///
///   let event = UserCreated {
///     id: 42,
///     name: "Jane Doe".to_string(),
///   };
///
///   publisher.publish(&event).await?;
///   Ok(())
/// }
/// ```
pub struct Pub<T, SerErr: EncErr + Send + Sync> {
  ctx: Arc<dyn PubCtxTrait + Send + Sync>,
  subject: String,
  encoder: Arc<dyn Encoder<Item = T, Error = SerErr> + Send + Sync>,
  _phantom: PhantomData<T>,
}

impl<T, SerErr> Pub<T, SerErr>
where
  T: Serialize + Send + Sync,
  SerErr: EncErr + Send + Sync,
{
  /// Creates a new publisher for the given subject and serialization format.
  ///
  /// # Parameters
  /// - `ctx`: Backend publish context that delivers serialized bytes.
  /// - `subject`: Destination subject or topic to send messages to.
  /// - `format`: Serialization format used when encoding published items.
  pub fn new(
    ctx: Arc<dyn PubCtxTrait + Send + Sync>,
    subject: impl Into<String>,
    encoder: Arc<dyn Encoder<Item = T, Error = SerErr> + Send + Sync>,
  ) -> Self {
    Self {
      ctx,
      subject: subject.into(),
      encoder,
      _phantom: PhantomData,
    }
  }
}

#[async_trait]
impl<T, SerErr> PubTrait for Pub<T, SerErr>
where
  T: Serialize + Send + Sync,
  SerErr: EncErr + Send + Sync,
{
  type Item = T;
  type EncodeErr = SerErr;
  /// Serializes the provided object and publishes it to the configured
  /// subject using the underlying context.
  ///
  /// # Parameters
  /// - `obj`: The typed value to encode and send to the subject.
  async fn publish(&self, obj: &T) -> Result<(), PubError<Self::EncodeErr>> {
    let payload = self.encoder.encode(obj).map_err(|e| EncodeError::new(e))?;
    self
      .ctx
      .publish(self.subject.as_str(), payload.into())
      .await?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use ::bytes::Bytes;
  use ::mockall::predicate::*;

  use crate::encoder::MockEncoder;
  use crate::errors::BrokerError;
  use crate::tests::entity::TestEntity;
  use crate::tests::error::MockBrokerErr;
  use crate::traits::MockPubCtxTrait;

  use super::*;

  #[tokio::test]
  async fn test_publish() {
    let entity = TestEntity::new(1, "Test Name");
    let subject = "test.subject";
    let correct = Bytes::from("serialized bytes");
    let mut ctx = MockPubCtxTrait::new();
    ctx
      .expect_publish()
      .with(eq(subject.clone()), eq(correct.clone()))
      .times(1)
      .returning(|_, _| Ok(()));
    let mut encoder = MockEncoder::new();
    encoder
      .expect_encode()
      .with(eq(entity.clone()))
      .times(1)
      .returning(move |_| Ok(correct.clone()));
    let publisher: Pub<TestEntity, _> =
      Pub::new(Arc::new(ctx), subject, Arc::new(encoder));
    let res = publisher.publish(&entity).await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn test_publish_error() {
    let entity = TestEntity::new(1, "Test Name");
    let subject = "test.subject.error";
    let correct = Bytes::from("serialized bytes");
    let mut ctx = MockPubCtxTrait::new();
    ctx
      .expect_publish()
      .with(eq(subject.clone()), eq(correct.clone()))
      .times(1)
      .returning(|_, _| Err(BrokerError::new(MockBrokerErr)));
    let mut encoder = MockEncoder::new();
    encoder
      .expect_encode()
      .with(eq(entity.clone()))
      .times(1)
      .returning(move |_| Ok(correct.clone()));
    let publisher: Pub<TestEntity, _> =
      Pub::new(Arc::new(ctx), subject, Arc::new(encoder));
    let res = publisher.publish(&entity).await;
    let err_msg = res.unwrap_err().to_string();
    assert_eq!(err_msg, MockBrokerErr {}.to_string());
  }
}

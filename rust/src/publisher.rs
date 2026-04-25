use ::std::marker::PhantomData;
use ::std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;
use serde::ser::Error as EncErr;

use crate::brokers::PubBrokerTrait;
use crate::encoder::Encoder;
use crate::errors::{EncodeError, PubError};
use crate::traits::PubTrait;

/// Publisher for serializable messages using a pluggable encoder and context.
///
/// The publisher encodes messages using the provided [`Encoder`]
/// and delegates the actual publish call to an injected [`PubBrokerTrait`] so it can
/// work with different backends. The encoder is passed at construction time, enabling
/// runtime format selection and supporting the "any-format" design.
///
/// # Example with JSON
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use serde::Serialize;
/// use object_transfer::{
///   encoder::JSONEncoder,
///   Pub,
///   traits::PubTrait,
/// };
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
///   let js = Arc::new(async_nats::jetstream::new(client));
///
///   // Create a publisher with JSON encoder
///   let publisher: Pub<UserCreated, _> = Pub::new(
///     js,
///     "events.user_created",
///     Arc::new(JSONEncoder::new()),
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
///
/// # Example with Custom Format
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use serde::Serialize;
/// use bytes::Bytes;
/// use object_transfer::{
///   encoder::Encoder,
///   Pub,
///   traits::PubTrait,
/// };
///
/// #[derive(Serialize)]
/// struct Message {
///   text: String,
/// }
///
/// struct CustomEncoder;
///
/// impl Encoder for CustomEncoder {
///   type Item = Message;
///   type Error = std::fmt::Error;
///
///   fn encode(&self, item: &Self::Item) -> Result<Bytes, Self::Error> {
///     Ok(Bytes::from(item.text.clone()))
///   }
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///   let client = async_nats::connect("demo.nats.io").await?;
///   let js = Arc::new(async_nats::jetstream::new(client));
///
///   let publisher: Pub<Message, _> = Pub::new(
///     js,
///     "events.custom",
///     Arc::new(CustomEncoder),
///   );
///
///   publisher.publish(&Message { text: "Hello".to_string() }).await?;
///   Ok(())
/// }
/// ```
pub struct Pub<T, SerErr: EncErr + Send + Sync> {
  ctx: Arc<dyn PubBrokerTrait + Send + Sync>,
  subject: String,
  encoder: Arc<dyn Encoder<Item = T, Error = SerErr> + Send + Sync>,
  _phantom: PhantomData<T>,
}

impl<T, SerErr> Pub<T, SerErr>
where
  T: Serialize + Send + Sync,
  SerErr: EncErr + Send + Sync,
{
  /// Creates a new publisher for the given subject with a pluggable encoder.
  ///
  /// # Parameters
  /// - `ctx`: Backend publish context that delivers serialized bytes.
  /// - `subject`: Destination subject or topic to send messages to.
  /// - `encoder`: A trait object implementing [`Encoder`] for your format.
  ///   Pass `Arc::new(JSONEncoder::new())`, `Arc::new(MessagePackEncoder::new())`, or
  ///   your custom encoder implementation.
  ///
  /// # Encoder Selection
  ///
  /// The encoder is passed at construction time, allowing for:
  /// - **Compile-time format selection**: Create different publisher instances with different types
  /// - **Runtime format selection**: Use `Arc<dyn Encoder<...>>` to select format dynamically
  ///
  /// # Error Types
  ///
  /// The generic `SerErr` type parameter is the error type of your encoder. Different encoders
  /// can use different error types (e.g., `serde_json::Error`, custom error types).
  pub fn new(
    ctx: Arc<dyn PubBrokerTrait + Send + Sync>,
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

  use crate::brokers::{errors::BrokerError, traits::MockPubBrokerTrait};
  use crate::encoder::MockEncoder;
  use crate::tests::entity::TestEntity;
  use crate::tests::error::{MockBrokerErr, MockEncErr};

  use super::*;

  #[tokio::test]
  async fn test_publish() {
    let entity = TestEntity::new(1, "Test Name");
    let subject = "test.subject";
    let correct = Bytes::from("serialized bytes");
    let mut ctx = MockPubBrokerTrait::new();
    ctx
      .expect_publish()
      .with(eq(subject), eq(correct.clone()))
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
    let mut ctx = MockPubBrokerTrait::new();
    ctx
      .expect_publish()
      .with(eq(subject), eq(correct.clone()))
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
    assert_eq!(
      err_msg,
      PubError::<MockEncErr>::BrokerError(BrokerError::new(MockBrokerErr))
        .to_string()
    );
  }
}

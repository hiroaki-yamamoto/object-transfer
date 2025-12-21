use ::std::marker::PhantomData;
use ::std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;

use crate::r#enum::Format;
use crate::error::PubError;
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
pub struct Pub<T> {
  ctx: Arc<dyn PubCtxTrait + Send + Sync>,
  subject: String,
  format: Format,
  _phantom: PhantomData<T>,
}

impl<T> Pub<T>
where
  T: Serialize + Send + Sync,
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
    format: Format,
  ) -> Self {
    Self {
      ctx,
      subject: subject.into(),
      format,
      _phantom: PhantomData,
    }
  }
}

#[async_trait]
impl<T> PubTrait for Pub<T>
where
  T: Serialize + Send + Sync,
{
  type Item = T;
  /// Serializes the provided object and publishes it to the configured
  /// subject using the underlying context.
  ///
  /// # Parameters
  /// - `obj`: The typed value to encode and send to the subject.
  async fn publish(&self, obj: &T) -> Result<(), PubError> {
    let payload = match self.format {
      Format::MessagePack => {
        rmp_serde::to_vec(obj).map_err(PubError::MessagePackEncode)?
      }
      Format::JSON => serde_json::to_vec(obj).map_err(PubError::Json)?,
    };
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
  use ::rmp_serde::to_vec as to_msgpack;
  use ::serde_json::to_vec as jsonify;

  use crate::r#enum::Format;
  use crate::tests::entity::TestEntity;
  use crate::traits::MockPubCtxTrait;

  use super::*;

  async fn test_publish(format: Format) {
    let entity = TestEntity::new(1, &format!("Test Name: {:?}", format));
    let subject = &format!("test.subject.{:?}", format);
    let correct = Bytes::from(match format {
      Format::MessagePack => to_msgpack(&entity).unwrap(),
      Format::JSON => jsonify(&entity).unwrap(),
    });
    let mut ctx = MockPubCtxTrait::new();
    ctx
      .expect_publish()
      .with(eq(subject.clone()), eq(correct))
      .times(1)
      .returning(|_, _| Ok(()));
    let publisher: Pub<TestEntity> = Pub::new(Arc::new(ctx), subject, format);
    let res = publisher.publish(&entity).await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn test_publish_json() {
    test_publish(Format::JSON).await;
  }

  #[tokio::test]
  async fn test_publish_msgpack() {
    test_publish(Format::MessagePack).await;
  }

  #[tokio::test]
  async fn test_publish_error() {
    let entity = TestEntity::new(1, "Test Name");
    let subject = "test.subject.error";
    let mut ctx = MockPubCtxTrait::new();
    ctx
      .expect_publish()
      .withf(move |subj, _| subj == subject)
      .times(1)
      .returning(|_, _| Err(PubError::ErrorTest));
    let publisher: Pub<TestEntity> =
      Pub::new(Arc::new(ctx), subject, Format::JSON);
    let res = publisher.publish(&entity).await;
    let err_msg = res.unwrap_err().to_string();
    assert_eq!(err_msg, PubError::ErrorTest.to_string());
  }
}

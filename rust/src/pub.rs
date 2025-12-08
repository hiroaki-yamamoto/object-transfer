use ::std::marker::PhantomData;
use ::std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;

use crate::r#enum::Format;
use crate::error::Error;
use crate::traits::{PubCtxTrait, PubTrait};

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
  pub fn new(
    ctx: Arc<dyn PubCtxTrait + Send + Sync>,
    subject: impl Into<String>,
    format: Format,
  ) -> Result<Self, Error> {
    Ok(Self {
      ctx,
      subject: subject.into(),
      format,
      _phantom: PhantomData,
    })
  }
}

#[async_trait]
impl<T> PubTrait for Pub<T>
where
  T: Serialize + Send + Sync,
{
  type Item = T;
  async fn publish(&self, obj: &T) -> Result<(), Error> {
    let payload = match self.format {
      Format::MessagePack => {
        rmp_serde::to_vec(obj).map_err(Error::MessagePackEncode)?
      }
      Format::JSON => serde_json::to_vec(obj).map_err(Error::Json)?,
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
    let publisher: Pub<TestEntity> =
      Pub::new(Arc::new(ctx), subject, format).unwrap();
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
      .returning(|_, _| Err(Error::ErrorTest));
    let publisher: Pub<TestEntity> =
      Pub::new(Arc::new(ctx), subject, Format::JSON).unwrap();
    let res = publisher.publish(&entity).await;
    let err_msg = res.unwrap_err().to_string();
    assert_eq!(err_msg, Error::ErrorTest.to_string());
  }
}

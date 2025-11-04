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
  pub async fn new(
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

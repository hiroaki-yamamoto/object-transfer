use ::std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;

use crate::r#enum::Format;
use crate::error::Error;
use crate::traits::{PubCtxTrait, PubTrait};

pub struct Pub {
  ctx: Arc<dyn PubCtxTrait + Send + Sync>,
  subject: String,
  format: Format,
}

impl Pub {
  pub async fn new(
    ctx: Arc<dyn PubCtxTrait + Send + Sync>,
    subject: impl Into<String>,
    format: Format,
  ) -> Result<Self, Error> {
    Ok(Self {
      ctx,
      subject: subject.into(),
      format,
    })
  }
}

#[async_trait]
impl PubTrait for Pub {
  async fn publish<T>(&self, obj: &T) -> Result<(), Error>
  where
    T: Serialize + Send + Sync,
  {
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

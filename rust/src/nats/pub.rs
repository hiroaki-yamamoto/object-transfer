use crate::{r#enum::Format, error::Error, traits::PubTrait};
use async_nats::jetstream;
use async_trait::async_trait;
use serde::Serialize;

#[derive(Debug)]
pub struct Pub {
  js: jetstream::Context,
  subject: String,
  format: Format,
}

impl Pub {
  pub async fn new(
    js: jetstream::Context,
    subject: impl Into<String>,
    format: Format,
  ) -> Result<Self, Error> {
    Ok(Self {
      js,
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
      .js
      .publish(self.subject.clone(), payload.into())
      .await?
      .await?;
    Ok(())
  }
}

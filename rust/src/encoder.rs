use ::bytes::Bytes;
use ::serde::{de::DeserializeOwned, ser::Serialize};
use ::std::error::Error;

pub trait Encoder {
  type Item: Serialize + Send + Sync;
  type Error: Error + Send + Sync;
  fn encode(&self, item: &Self::Item) -> Result<Bytes, Box<Self::Error>>;
}

pub trait Decoder {
  type Item: DeserializeOwned + Send + Sync;
  type Error: Error + Send + Sync;
  fn decode(&self, data: Bytes) -> Result<Self::Item, Box<Self::Error>>;
}

#[cfg(test)]
mod test {
  use ::static_assertions::assert_obj_safe;
  use ::std::io::Error as IoError;

  use super::*;
  use crate::tests::entity::TestEntity;
  #[test]
  fn test_encoder_safety() {
    assert_obj_safe!(Encoder<Item = TestEntity, Error = IoError>);
  }

  #[test]
  fn test_decoder_safety() {
    assert_obj_safe!(Decoder<Item = TestEntity, Error = IoError>);
  }
}

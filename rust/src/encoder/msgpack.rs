use ::std::marker::PhantomData;

use ::bytes::Bytes;
use ::rmp_serde::{
  decode::{Error as DecodeError, from_slice},
  encode::{Error as EncodeError, to_vec},
};
use ::serde::{de::DeserializeOwned, ser::Serialize};

use super::traits::{Decoder, Encoder};

pub struct MessagePackEncoder<T: Serialize + Send + Sync> {
  _marker: PhantomData<T>,
}

impl<T: Serialize + Send + Sync> MessagePackEncoder<T> {
  pub fn new() -> Self {
    Self {
      _marker: PhantomData,
    }
  }
}

impl<T: Serialize + Send + Sync> Encoder for MessagePackEncoder<T> {
  type Item = T;
  type Error = EncodeError;

  fn encode(&self, item: &Self::Item) -> Result<Bytes, Self::Error> {
    Ok(Bytes::from(to_vec(item)?))
  }
}

pub struct MessagePackDecoder<T: DeserializeOwned + Send + Sync> {
  _marker: PhantomData<T>,
}

impl<T: DeserializeOwned + Send + Sync> MessagePackDecoder<T> {
  pub fn new() -> Self {
    Self {
      _marker: PhantomData,
    }
  }
}

impl<T: DeserializeOwned + Send + Sync> Decoder for MessagePackDecoder<T> {
  type Item = T;
  type Error = DecodeError;

  fn decode(&self, data: Bytes) -> Result<Self::Item, Self::Error> {
    from_slice(&data)
  }
}

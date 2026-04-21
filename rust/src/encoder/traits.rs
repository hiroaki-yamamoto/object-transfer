//! Traits for encoding and decoding data types.
//!
//! This module provides the core trait abstractions for serializing Rust types
//! into a byte representation and deserializing byte data back into Rust types.
//!
//! # Traits
//!
//! - [`Encoder`]: Encodes items into byte sequences using serde serialization.
//! - [`Decoder`]: Decodes byte sequences back into items using serde deserialization.
//!
//! Both traits are object-safe, allowing for dynamic dispatch through trait objects.
//! They require associated types to implement `Send + Sync` for thread-safe usage.

use ::bytes::Bytes;
use ::serde::{de::DeserializeOwned, ser::Serialize};
use ::std::error::Error;

/// A trait for encoding items into byte sequences.
///
/// `Encoder` defines the interface for serializing Rust types into their byte representation.
/// Implementations use serde for serialization and the trait is object-safe, supporting dynamic
/// dispatch through trait objects.
///
/// # Associated Types
///
/// * `Item` - The type being encoded. Must implement [`Serialize`](serde::ser::Serialize) and be
///   `Send + Sync` for thread-safe usage.
/// * `Error` - The error type returned when encoding fails. Must implement
///   [`Error`](std::error::Error) and be `Send + Sync`.
///
/// # Methods
///
/// * [`encode`](Self::encode) - Serializes an item into a byte sequence.
///
/// # Examples
///
/// Implementing a JSON encoder:
///
/// ```
/// use bytes::Bytes;
/// use serde::Serialize;
/// use std::error::Error;
///
/// use object_transfer::encoder::Encoder;
///
/// #[derive(Serialize)]
/// struct MyType {
///    id: u32,
/// }
///
/// struct JsonEncoder;
///
/// impl Encoder for JsonEncoder {
///     type Item = MyType;
///     type Error = serde_json::Error;
///
///     fn encode(&self, item: &Self::Item) -> Result<Bytes, Box<Self::Error>> {
///         let json = serde_json::to_string(item)?;
///         Ok(Bytes::from(json))
///     }
/// }
/// ```
pub trait Encoder {
  type Item: Serialize + Send + Sync;
  type Error: Error + Send + Sync;
  fn encode(&self, item: &Self::Item) -> Result<Bytes, Box<Self::Error>>;
}

/// A trait for decoding byte sequences back into items.
///
/// `Decoder` defines the interface for deserializing byte data back into Rust types.
/// Implementations use serde for deserialization and the trait is object-safe, supporting dynamic
/// dispatch through trait objects.
///
/// # Associated Types
///
/// * `Item` - The type being decoded. Must implement [`DeserializeOwned`](serde::de::DeserializeOwned) and be
///   `Send + Sync` for thread-safe usage.
/// * `Error` - The error type returned when decoding fails. Must implement
///   [`Error`](std::error::Error) and be `Send + Sync`.
///
/// # Methods
///
/// * [`decode`](Self::decode) - Deserializes a byte sequence into an item.
///
/// # Examples
///
/// Implementing a JSON decoder:
///
/// ```
/// use std::error::Error;
///
/// use bytes::Bytes;
/// use serde::de::{DeserializeOwned};
/// use serde::Deserialize;
/// use object_transfer::encoder::Decoder;
///
/// #[derive(Deserialize)]
/// struct MyType {
///     id: u32,
/// }
///
/// struct JsonDecoder;
///
/// impl Decoder for JsonDecoder {
///     type Item = MyType;
///     type Error = serde_json::Error;
///
///     fn decode(&self, data: Bytes) -> Result<Self::Item, Box<Self::Error>> {
///         let item = serde_json::from_slice(&data)?;
///         Ok(item)
///     }
/// }
/// ```
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

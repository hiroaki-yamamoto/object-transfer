//! JSON encoding and decoding for serializable types.
//!
//! This module provides generic implementations for encoding Rust types into JSON format
//! and decoding JSON data back into Rust types. Both implementations use `serde_json` for
//! serialization and deserialization.
//!
//! # Types
//!
//! - [`JSONEncoder`]: Encodes items into JSON byte sequences. Implements [`Encoder`].
//! - [`JSONDecoder`]: Decodes JSON byte sequences into items. Implements [`Decoder`].
//!
//! # Example
//!
//! ```
//! use serde::{Serialize, Deserialize};
//! use object_transfer::encoder::{JSONEncoder, JSONDecoder, Encoder, Decoder};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Message {
//!     id: u32,
//!     content: String,
//! }
//!
//! fn example() {
//!   let encoder = JSONEncoder::new();
//!   let message = Message { id: 1, content: "Hello".to_string() };
//!   let bytes = encoder.encode(&message).expect("Failed to encode");
//!
//!   let decoder = JSONDecoder::new();
//!   let decoded: Message = decoder.decode(bytes).expect("Failed to decode");
//!   assert_eq!(decoded.id, 1);
//! }
//! ```

use ::std::marker::PhantomData;

use ::bytes::Bytes;
use ::serde::{de::DeserializeOwned, ser::Serialize};
use ::serde_json::{Error as JSErr, from_slice, to_vec};

use super::traits::{Decoder, Encoder};

/// Encodes items into JSON byte sequences.
///
/// `JSONEncoder` is a generic encoder that converts any serializable type into JSON-formatted bytes.
/// It implements the [`Encoder`] trait and uses `serde_json` for serialization.
///
/// # Type Parameters
///
/// * `T` - The type to be encoded. Must implement [`serde::Serialize`], [`Send`], and [`Sync`].
///
/// # Examples
///
/// ```
/// use serde::Serialize;
/// use object_transfer::encoder::{JSONEncoder, Encoder};
///
/// #[derive(Serialize)]
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// fn example() {
///   let encoder = JSONEncoder::new();
///   let user = User {
///       id: 42,
///       name: "Alice".to_string(),
///   };
///   let bytes = encoder.encode(&user).expect("failed to encode");
///   assert!(!bytes.is_empty());
/// }
/// ```
///
/// # Errors
///
/// Encoding fails if serialization encounters an error, such as when the type contains
/// non-serializable fields or when the encoder runs out of memory.
pub struct JSONEncoder<T: Serialize + Send + Sync> {
  _marker: PhantomData<T>,
}

impl<T: Serialize + Send + Sync> JSONEncoder<T> {
  /// Creates a new instance of `JSONEncoder`.
  pub fn new() -> Self {
    Self {
      _marker: PhantomData,
    }
  }
}

impl<T: Serialize + Send + Sync> Encoder for JSONEncoder<T> {
  type Item = T;
  type Error = JSErr;

  fn encode(&self, item: &Self::Item) -> Result<bytes::Bytes, Self::Error> {
    let payload = to_vec(item)?;
    Ok(Bytes::from(payload))
  }
}

/// Decodes JSON byte sequences into items.
///
/// `JSONDecoder` is a generic decoder that converts JSON-formatted bytes back into any deserializable type.
/// It implements the [`Decoder`] trait and uses `serde_json` for deserialization.
///
/// # Type Parameters
///
/// * `T` - The type to be decoded. Must implement [`serde::de::DeserializeOwned`], [`Send`], and [`Sync`].
///
/// # Examples
///
/// ```
/// use serde::Deserialize;
/// use object_transfer::encoder::{JSONDecoder, Decoder};
/// use bytes::Bytes;
///
/// #[derive(Deserialize)]
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// fn example() {
///   let decoder = JSONDecoder::new();
///   let json = r#"{"id": 42, "name": "Alice"}"#;
///   let user: User = decoder.decode(Bytes::from(json))
///       .expect("failed to decode");
///   assert_eq!(user.id, 42);
///   assert_eq!(user.name, "Alice");
/// }
/// ```
///
/// # Errors
///
/// Decoding fails if deserialization encounters an error, such as:
/// - Invalid JSON syntax in the payload
/// - Type mismatch between the JSON structure and the target type `T`
/// - Missing or extra fields not matching the target type's requirements
pub struct JSONDecoder<T: DeserializeOwned + Send + Sync> {
  _marker: PhantomData<T>,
}

impl<T: DeserializeOwned + Send + Sync> JSONDecoder<T> {
  /// Creates a new instance of `JSONDecoder`.
  pub fn new() -> Self {
    Self {
      _marker: PhantomData,
    }
  }
}

impl<T: DeserializeOwned + Send + Sync> Decoder for JSONDecoder<T> {
  type Item = T;
  type Error = JSErr;

  fn decode(&self, payload: Bytes) -> Result<Self::Item, Self::Error> {
    let item = from_slice(&payload)?;
    Ok(item)
  }
}

#[cfg(test)]
mod test {
  use ::serde::{Deserialize, Serialize};

  use crate::tests::entity::TestEntity;

  use super::*;

  #[test]
  fn test_json_encoder_basic() {
    let encoder = JSONEncoder::new();

    let data = TestEntity::new(1, "test");

    let result = encoder.encode(&data);
    assert!(result.is_ok());

    let bytes = result.unwrap();
    assert!(!bytes.is_empty());
  }

  #[test]
  fn test_json_decoder_basic() {
    let decoder = JSONDecoder::new();

    let json = br#"{"id":42,"name":"example"}"#;
    let bytes = Bytes::copy_from_slice(json);

    let result = decoder.decode(bytes);
    assert!(result.is_ok());

    let data: TestEntity = result.unwrap();
    assert_eq!(data.id, 42);
    assert_eq!(data.name, "example");
  }

  #[test]
  fn test_encode_decode_roundtrip() {
    let encoder = JSONEncoder::new();
    let decoder = JSONDecoder::new();

    let original = TestEntity::new(123, "roundtrip_test");

    // Encode
    let encoded = encoder.encode(&original).expect("encoding failed");

    // Decode
    let decoded: TestEntity =
      decoder.decode(encoded).expect("decoding failed");

    // Verify equality
    assert_eq!(decoded, original);
  }

  #[test]
  fn test_json_encoder_multiple_types() {
    let encoder_string = JSONEncoder::<String> {
      _marker: PhantomData,
    };

    let result = encoder_string.encode(&"hello world".to_string());
    assert!(result.is_ok());
  }

  #[test]
  fn test_json_decoder_invalid_json() {
    let decoder = JSONDecoder::<TestEntity> {
      _marker: PhantomData,
    };

    let invalid_json = Bytes::from_static(b"not valid json");
    let result = decoder.decode(invalid_json);

    assert!(result.is_err());
  }

  #[test]
  fn test_json_decoder_type_mismatch() {
    let decoder = JSONDecoder::<TestEntity> {
      _marker: PhantomData,
    };

    let json = br#"{"id":"not_a_number","name":"test"}"#;
    let bytes = Bytes::copy_from_slice(json);

    let result = decoder.decode(bytes);
    assert!(result.is_err());
  }

  #[test]
  fn test_encode_decode_with_nested_structure() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Nested {
      data: TestEntity,
      metadata: String,
    }

    let encoder = JSONEncoder::<Nested> {
      _marker: PhantomData,
    };
    let decoder = JSONDecoder::<Nested> {
      _marker: PhantomData,
    };

    let original = Nested {
      data: TestEntity::new(999, "nested"),
      metadata: "test metadata".to_string(),
    };

    let encoded = encoder.encode(&original).expect("encoding failed");
    let decoded = decoder.decode(encoded).expect("decoding failed");

    assert_eq!(decoded, original);
  }
}

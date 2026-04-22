use ::std::marker::PhantomData;

use ::bytes::Bytes;
use ::serde::{de::DeserializeOwned, ser::Serialize};
use ::serde_json::{Error as JSErr, from_slice, to_vec};

use super::traits::{Decoder as DecoderTrait, Encoder as EncoderTrait};

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
/// use object_transfer::encoder::{Encoder, JSONEncoder};
///
/// #[derive(Serialize)]
/// struct User {
///     id: u32,
///     name: String,
/// }
///
/// fn example() {
///   let encoder = Encoder::new();
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
#[derive(Debug)]
pub struct Encoder<T: Serialize + Send + Sync> {
  _marker: PhantomData<T>,
}

impl<T: Serialize + Send + Sync> Encoder<T> {
  /// Creates a new instance of `Encoder`.
  pub fn new() -> Self {
    Self {
      _marker: PhantomData,
    }
  }
}

impl<T: Serialize + Send + Sync> EncoderTrait for Encoder<T> {
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
#[derive(Debug)]
pub struct Decoder<T: DeserializeOwned + Send + Sync> {
  _marker: PhantomData<T>,
}

impl<T: DeserializeOwned + Send + Sync> Decoder<T> {
  /// Creates a new instance of `Decoder`.
  pub fn new() -> Self {
    Self {
      _marker: PhantomData,
    }
  }
}

impl<T: DeserializeOwned + Send + Sync> DecoderTrait for Decoder<T> {
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
    let encoder = Encoder::new();

    let data = TestEntity::new(1, "test");

    let result = encoder.encode(&data);
    assert!(result.is_ok());

    let bytes = result.unwrap();
    assert!(!bytes.is_empty());
  }

  #[test]
  fn test_json_decoder_basic() {
    let decoder = Decoder::new();

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
    let encoder = Encoder::new();
    let decoder = Decoder::new();

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
    let encoder_string = Encoder::<String>::new();

    let result = encoder_string.encode(&"hello world".to_string());
    assert!(result.is_ok());
  }

  #[test]
  fn test_json_decoder_invalid_json() {
    let decoder = Decoder::<TestEntity>::new();

    let invalid_json = Bytes::from_static(b"not valid json");
    let result = decoder.decode(invalid_json);

    assert!(result.is_err());
  }

  #[test]
  fn test_json_decoder_type_mismatch() {
    let decoder = Decoder::<TestEntity>::new();

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

    let encoder = Encoder::<Nested>::new();
    let decoder = Decoder::<Nested>::new();

    let original = Nested {
      data: TestEntity::new(999, "nested"),
      metadata: "test metadata".to_string(),
    };

    let encoded = encoder.encode(&original).expect("encoding failed");
    let decoded = decoder.decode(encoded).expect("decoding failed");

    assert_eq!(decoded, original);
  }
}

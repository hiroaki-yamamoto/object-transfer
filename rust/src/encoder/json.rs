use ::std::marker::PhantomData;

use ::bytes::Bytes;
use ::serde::{de::DeserializeOwned, ser::Serialize};
use ::serde_json::{Error as JSErr, to_vec};

use super::traits::{Decoder, Encoder};

pub struct JSONEncoder<T: Serialize + Send + Sync> {
  _marker: PhantomData<T>,
}

impl<T: Serialize + Send + Sync> Encoder for JSONEncoder<T> {
  type Item = T;
  type Error = JSErr;

  fn encode(
    &self,
    item: &Self::Item,
  ) -> Result<bytes::Bytes, Box<Self::Error>> {
    let payload = to_vec(item)?;
    Ok(Bytes::from(payload))
  }
}

pub struct JSONDecoder<T: DeserializeOwned + Send + Sync> {
  _marker: PhantomData<T>,
}

impl<T: DeserializeOwned + Send + Sync> Decoder for JSONDecoder<T> {
  type Item = T;
  type Error = JSErr;

  fn decode(&self, payload: Bytes) -> Result<Self::Item, Box<Self::Error>> {
    let item = serde_json::from_slice(&payload)?;
    Ok(item)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use ::serde::{Deserialize, Serialize};

  #[derive(Debug, Serialize, Deserialize, PartialEq)]
  struct TestData {
    id: u32,
    name: String,
    active: bool,
  }

  #[test]
  fn test_json_encoder_basic() {
    let encoder = JSONEncoder::<TestData> {
      _marker: PhantomData,
    };

    let data = TestData {
      id: 1,
      name: "test".to_string(),
      active: true,
    };

    let result = encoder.encode(&data);
    assert!(result.is_ok());

    let bytes = result.unwrap();
    assert!(!bytes.is_empty());
  }

  #[test]
  fn test_json_decoder_basic() {
    let decoder = JSONDecoder::<TestData> {
      _marker: PhantomData,
    };

    let json = br#"{"id":42,"name":"example","active":false}"#;
    let bytes = Bytes::copy_from_slice(json);

    let result = decoder.decode(bytes);
    assert!(result.is_ok());

    let data = result.unwrap();
    assert_eq!(data.id, 42);
    assert_eq!(data.name, "example");
    assert!(!data.active);
  }

  #[test]
  fn test_encode_decode_roundtrip() {
    let encoder = JSONEncoder::<TestData> {
      _marker: PhantomData,
    };
    let decoder = JSONDecoder::<TestData> {
      _marker: PhantomData,
    };

    let original = TestData {
      id: 123,
      name: "roundtrip_test".to_string(),
      active: true,
    };

    // Encode
    let encoded = encoder.encode(&original).expect("encoding failed");

    // Decode
    let decoded = decoder.decode(encoded).expect("decoding failed");

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
    let decoder = JSONDecoder::<TestData> {
      _marker: PhantomData,
    };

    let invalid_json = Bytes::from_static(b"not valid json");
    let result = decoder.decode(invalid_json);

    assert!(result.is_err());
  }

  #[test]
  fn test_json_decoder_type_mismatch() {
    let decoder = JSONDecoder::<TestData> {
      _marker: PhantomData,
    };

    let json = br#"{"id":"not_a_number","name":"test","active":true}"#;
    let bytes = Bytes::copy_from_slice(json);

    let result = decoder.decode(bytes);
    assert!(result.is_err());
  }

  #[test]
  fn test_encode_decode_with_nested_structure() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Nested {
      data: TestData,
      metadata: String,
    }

    let encoder = JSONEncoder::<Nested> {
      _marker: PhantomData,
    };
    let decoder = JSONDecoder::<Nested> {
      _marker: PhantomData,
    };

    let original = Nested {
      data: TestData {
        id: 999,
        name: "nested".to_string(),
        active: false,
      },
      metadata: "test metadata".to_string(),
    };

    let encoded = encoder.encode(&original).expect("encoding failed");
    let decoded = decoder.decode(encoded).expect("decoding failed");

    assert_eq!(decoded, original);
  }
}

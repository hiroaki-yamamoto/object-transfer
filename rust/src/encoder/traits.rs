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
use ::serde::{
  de::{DeserializeOwned, Error as DecodeErr},
  ser::{Error as EncodeErr, Serialize},
};

#[cfg(test)]
use ::mockall::automock;

#[cfg(test)]
use crate::tests::{
  entity::TestEntity,
  error::{MockDeErr, MockEncErr},
};

/// A trait for encoding items into byte sequences.
///
/// `Encoder` defines the interface for serializing Rust types into their byte representation.
/// The trait is object-safe, supporting dynamic dispatch through trait objects, which enables
/// runtime format selection—the core of the library's "any-format" design.
///
/// # Associated Types
///
/// * `Item` - The type being encoded. Must implement [`Serialize`] and be
///   `Send + Sync` for thread-safe usage.
/// * `Error` - The error type returned when encoding fails. Must implement
///   [`Error`](std::error::Error) and be `Send + Sync`. Each format can define its own error type
///   (e.g., `serde_json::Error`, `rmp_serde::Error`).
///
/// # Methods
///
/// * [`encode`](Self::encode) - Serializes an item into a byte sequence.
///
/// # Design Philosophy
///
/// Rather than restricting you to a fixed set of formats, this library lets you implement
/// `Encoder` for any serialization format. Built-in implementations (JSON, MessagePack) are
/// provided as examples, but you can add CBOR, Protocol Buffers, custom binary formats, or anything else.
///
/// # Examples
///
/// Implementing a custom plain-text encoder:
///
/// ```rust
/// use bytes::Bytes;
/// use serde::Serialize;
/// use object_transfer::encoder::Encoder;
///
/// #[derive(Serialize)]
/// struct MyType {
///    id: u32,
///    name: String,
/// }
///
/// struct PlainTextEncoder;
///
/// impl Encoder for PlainTextEncoder {
///     type Item = MyType;
///     type Error = std::fmt::Error;
///
///     fn encode(&self, item: &Self::Item) -> Result<Bytes, Self::Error> {
///         let text = format!("id={},name={}", item.id, item.name);
///         Ok(Bytes::from(text))
///     }
/// }
/// ```
///
/// Using your custom encoder with [`Pub`](crate::Pub):
///
/// ```rust,no_run
/// # use bytes::Bytes;
/// # use serde::Serialize;
/// # use object_transfer::encoder::Encoder;
/// # #[derive(Serialize)]
/// # struct MyType { id: u32, name: String }
/// # struct PlainTextEncoder;
/// # impl Encoder for PlainTextEncoder {
/// #     type Item = MyType;
/// #     type Error = serde_json::Error;
/// #     fn encode(&self, item: &Self::Item) -> Result<Bytes, Self::Error> {
/// #         serde_json::to_vec(item).map(|v| Bytes::from(v))
/// #     }
/// # }
/// use std::sync::Arc;
/// use object_transfer::{Pub, traits::PubTrait};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = async_nats::connect("demo.nats.io").await?;
///     let js = Arc::new(async_nats::jetstream::new(client));
///
///     let publisher: Pub<MyType, _> = Pub::new(
///         js,
///         "events",
///         Arc::new(PlainTextEncoder),
///     );
///
///     publisher.publish(&MyType { id: 1, name: "test".to_string() }).await?;
///     Ok(())
/// }
/// ```
#[cfg_attr(test, automock(type Item = TestEntity; type Error = MockEncErr;))]
pub trait Encoder {
  type Item: Serialize + Send + Sync;
  type Error: EncodeErr + Send + Sync;
  fn encode(&self, item: &Self::Item) -> Result<Bytes, Self::Error>;
}

/// A trait for decoding byte sequences back into items.
///
/// `Decoder` defines the interface for deserializing byte data back into Rust types.
/// The trait is object-safe, supporting dynamic dispatch through trait objects, enabling
/// runtime format selection paired with [`Encoder`].
///
/// # Associated Types
///
/// * `Item` - The type being decoded. Must implement [`DeserializeOwned`] and be
///   `Send + Sync` for thread-safe usage.
/// * `Error` - The error type returned when decoding fails. Must implement
///   [`Error`](std::error::Error) and be `Send + Sync`. Each format can define its own error type
///   (e.g., `serde_json::Error`, custom parse errors).
///
/// # Methods
///
/// * [`decode`](Self::decode) - Deserializes a byte sequence into an item.
///
/// # Design Philosophy
///
/// Like [`Encoder`], this trait supports any deserialization format. Implement it alongside
/// [`Encoder`] to create custom serialization formats.
///
/// # Examples
///
/// Implementing a custom plain-text decoder:
///
/// ```rust
/// use bytes::Bytes;
/// use serde::Deserialize;
/// use object_transfer::encoder::Decoder;
/// use std::num::ParseIntError;
///
/// #[derive(Deserialize)]
/// struct MyType {
///     id: u32,
///     name: String,
/// }
///
/// #[derive(Debug)]
/// enum ParseError {
///     InvalidFormat,
///     ParseInt(ParseIntError),
/// }
///
/// impl std::fmt::Display for ParseError {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         match self {
///             ParseError::InvalidFormat => write!(f, "Invalid format"),
///             ParseError::ParseInt(e) => write!(f, "Parse error: {}", e),
///         }
///     }
/// }
///
/// impl std::error::Error for ParseError {}
///
/// impl serde::de::Error for ParseError {
///     fn custom<T: std::fmt::Display>(_msg: T) -> Self {
///         ParseError::InvalidFormat
///     }
/// }
///
/// struct PlainTextDecoder;
///
/// impl Decoder for PlainTextDecoder {
///     type Item = MyType;
///     type Error = ParseError;
///
///     fn decode(&self, data: Bytes) -> Result<Self::Item, Self::Error> {
///         let text = String::from_utf8(data.to_vec())
///             .map_err(|_| ParseError::InvalidFormat)?;
///         let parts: Vec<&str> = text.split(',').collect();
///         if parts.len() != 2 {
///             return Err(ParseError::InvalidFormat);
///         }
///         let id = parts[0]
///             .strip_prefix("id=")
///             .ok_or(ParseError::InvalidFormat)?
///             .parse::<u32>()
///             .map_err(ParseError::ParseInt)?;
///         let name = parts[1]
///             .strip_prefix("name=")
///             .ok_or(ParseError::InvalidFormat)?
///             .to_string();
///         Ok(MyType { id, name })
///     }
/// }
/// ```
///
/// Using your custom decoder with [`Sub`](crate::Sub):
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use futures::StreamExt;
/// use object_transfer::{Sub, SubOpt, traits::SubTrait};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = async_nats::connect("demo.nats.io").await?;
///     let js = Arc::new(async_nats::jetstream::new(client));
///
///     // Note: This example skips fetcher setup for brevity
///     // See the Sub documentation for a complete example
///     Ok(())
/// }
/// ```

#[cfg_attr(test, automock(type Item = TestEntity; type Error = MockDeErr;))]
pub trait Decoder {
  type Item: DeserializeOwned + Send + Sync;
  type Error: DecodeErr + Send + Sync;
  fn decode(&self, data: Bytes) -> Result<Self::Item, Self::Error>;
}

#[cfg(test)]
mod test {
  use ::static_assertions::assert_obj_safe;

  use super::*;
  use crate::tests::entity::TestEntity;
  #[test]
  fn test_encoder_safety() {
    assert_obj_safe!(Encoder<Item = TestEntity, Error = MockEncErr>);
  }

  #[test]
  fn test_decoder_safety() {
    assert_obj_safe!(Decoder<Item = TestEntity, Error = MockDeErr>);
  }
}

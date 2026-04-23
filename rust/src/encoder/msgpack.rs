//! MessagePack encoder and decoder implementations.
//!
//! This module provides `Encoder` and `Decoder` trait implementations for the
//! MessagePack binary serialization format. MessagePack is a space-efficient
//! alternative to JSON that offers faster serialization and smaller payload sizes.
//!
//! # Example
//!
//! ```rust
//! use object_transfer::encoder::{Encoder, Decoder};
//! use object_transfer::encoder::msgpack::{
//!   Encoder as MsgPackEncoder,
//!   Decoder as MsgPackDecoder
//! };
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Message {
//!     id: u32,
//!     content: String,
//! }
//!
//! let encoder = MsgPackEncoder::new();
//! let decoder = MsgPackDecoder::new();
//!
//! let msg = Message { id: 1, content: "Hello".to_string() };
//! let encoded = encoder.encode(&msg)?;
//! let decoded: Message = decoder.decode(encoded)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use ::std::marker::PhantomData;

use ::bytes::Bytes;
use ::rmp_serde::{
  decode::{Error as DecodeError, from_slice},
  encode::{Error as EncodeError, to_vec},
};
use ::serde::{de::DeserializeOwned, ser::Serialize};

use super::traits::{Decoder as DecoderTrait, Encoder as EncoderTrait};

/// A MessagePack encoder for serializing data structures to MessagePack format.
///
/// `Encoder<T>` implements the [`Encoder`](super::traits::Encoder) trait to provide
/// MessagePack serialization for any type `T` that implements [`serde::Serialize`].
///
/// # Type Parameters
///
/// * `T` - The data type to be encoded. Must implement [`Serialize`], [`Send`], and [`Sync`].
///
/// # Example
///
/// ```rust
/// use object_transfer::encoder::Encoder;
/// use object_transfer::encoder::msgpack::{
///   Encoder as MsgPackEncoder,
///   Decoder as MsgPackDecoder
/// };
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Event {
///     id: u32,
///     message: String,
/// }
///
/// let encoder = MsgPackEncoder::new();
/// let event = Event { id: 1, message: "Hello".to_string() };
/// let encoded = encoder.encode(&event)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Implementation Details
///
/// The encoder uses the [`rmp_serde`] crate to perform the actual MessagePack
/// serialization. It returns a [`Bytes`] buffer containing the encoded data,
/// or an [`EncodeError`](::rmp_serde::encode::Error) if serialization fails.
#[derive(Debug)]
pub struct Encoder<T: Serialize + Send + Sync> {
  _marker: PhantomData<T>,
}

impl<T: Serialize + Send + Sync> Encoder<T> {
  /// Creates a new MessagePack encoder.
  ///
  /// # Example
  ///
  /// ```rust
  /// use object_transfer::encoder::msgpack::Encoder as MsgPackEncoder;
  /// use serde::Serialize;
  ///
  /// #[derive(Serialize)]
  /// struct Data {
  ///     value: i32,
  /// }
  ///
  /// let encoder: MsgPackEncoder<Data> = MsgPackEncoder::new();
  /// ```
  pub fn new() -> Self {
    Self {
      _marker: PhantomData,
    }
  }
}

impl<T: Serialize + Send + Sync> EncoderTrait for Encoder<T> {
  type Item = T;
  type Error = EncodeError;

  /// Encodes a value into MessagePack format.
  ///
  /// # Arguments
  ///
  /// * `item` - A reference to the value to be encoded
  ///
  /// # Returns
  ///
  /// Returns a [`Bytes`] buffer containing the MessagePack-encoded data on success,
  /// or an [`EncodeError`](::rmp_serde::encode::Error) if serialization fails.
  ///
  /// # Example
  ///
  /// ```rust
  /// use object_transfer::encoder::{Encoder, Encoder as EncoderTrait};
  /// use object_transfer::encoder::msgpack::{
  ///   Encoder as MsgPackEncoder,
  ///   Decoder as MsgPackDecoder
  /// };
  /// use serde::Serialize;
  ///
  /// #[derive(Serialize)]
  /// struct Message {
  ///     id: u32,
  ///     text: String,
  /// }
  ///
  /// let encoder = MsgPackEncoder::new();
  /// let msg = Message { id: 1, text: "hello".to_string() };
  /// let encoded = encoder.encode(&msg)?;
  /// # Ok::<(), Box<dyn std::error::Error>>(())
  /// ```
  fn encode(&self, item: &Self::Item) -> Result<Bytes, Self::Error> {
    Ok(Bytes::from(to_vec(item)?))
  }
}

/// A MessagePack decoder for deserializing data structures from MessagePack format.
///
/// `Decoder<T>` implements the [`Decoder`](super::traits::Decoder) trait to provide
/// MessagePack deserialization for any type `T` that implements [`serde::Deserialize`].
///
/// # Type Parameters
///
/// * `T` - The data type to be decoded. Must implement [`DeserializeOwned`], [`Send`], and [`Sync`].
///
/// # Example
///
/// ```rust
/// use object_transfer::encoder::{Decoder, Encoder};
/// use object_transfer::encoder::msgpack::{
///   Encoder as MsgPackEncoder,
///   Decoder as MsgPackDecoder
/// };
/// use serde::{Serialize, Deserialize};
/// use bytes::Bytes;
///
/// #[derive(Serialize, Deserialize)]
/// struct Event {
///     id: u32,
///     message: String,
/// }
///
/// let encoder = MsgPackEncoder::new();
/// let decoder = MsgPackDecoder::new();
///
/// let event = Event { id: 42, message: "Hello".to_string() };
/// let encoded = encoder.encode(&event)?;
/// let decoded: Event = decoder.decode(encoded)?;
/// assert_eq!(decoded.id, 42);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Implementation Details
///
/// The decoder uses the [`rmp_serde`] crate to perform the actual MessagePack
/// deserialization. It accepts a [`Bytes`] buffer and returns the decoded value,
/// or a [`DecodeError`](::rmp_serde::decode::Error) if deserialization fails.
#[derive(Debug)]
pub struct Decoder<T: DeserializeOwned + Send + Sync> {
  _marker: PhantomData<T>,
}

impl<T: DeserializeOwned + Send + Sync> Decoder<T> {
  /// Creates a new MessagePack decoder.
  ///
  /// # Example
  ///
  /// ```rust
  /// use object_transfer::encoder::msgpack::Decoder as MsgPackDecoder;
  /// use serde::Deserialize;
  ///
  /// #[derive(Deserialize)]
  /// struct Data {
  ///     value: i32,
  /// }
  ///
  /// let decoder: MsgPackDecoder<Data> = MsgPackDecoder::new();
  /// ```
  pub fn new() -> Self {
    Self {
      _marker: PhantomData,
    }
  }
}

impl<T: DeserializeOwned + Send + Sync> DecoderTrait for Decoder<T> {
  type Item = T;
  type Error = DecodeError;

  fn decode(&self, data: Bytes) -> Result<Self::Item, Self::Error> {
    from_slice(&data)
  }
}

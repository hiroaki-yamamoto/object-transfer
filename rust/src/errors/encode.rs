//! Encoding error types and conversions.
//!
//! This module provides the [`EncodeError`] type for representing errors that occur during
//! serialization of messages. It includes implementations to convert from various serialization
//! format errors (JSON via serde_json and MessagePack via rmp_serde) into a unified error type.

use ::serde::ser::Error as EncErr;

use ::thiserror::Error;

/// Error type for encoding/serialization failures.
///
/// This struct provides a unified error type for encoding errors that may originate from
/// different serialization formats (e.g., JSON via `serde_json`, MessagePack via `rmp_serde`).
/// The underlying error is boxed to allow for flexible error handling across different encoding
/// implementations.
#[derive(Error, Debug)]
#[error("encoding error: {kind}")]
pub struct EncodeError<E: EncErr + Send + Sync> {
  kind: E,
}

impl<E: EncErr + Send + Sync> EncodeError<E> {
  /// Creates a new `EncodeError` from any error that implements `std::error::Error`.
  pub(crate) fn new(err: E) -> Self {
    EncodeError { kind: err }
  }
}

#[cfg(feature = "json")]
impl From<serde_json::Error> for EncodeError<serde_json::Error> {
  fn from(err: serde_json::Error) -> Self {
    EncodeError { kind: err }
  }
}

#[cfg(feature = "msgpack")]
impl From<rmp_serde::encode::Error> for EncodeError<rmp_serde::encode::Error> {
  fn from(err: rmp_serde::encode::Error) -> Self {
    EncodeError { kind: err }
  }
}

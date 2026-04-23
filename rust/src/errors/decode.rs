//! Decoding error types for object deserialization.
//!
//! This module provides error handling for decoding failures that occur during
//! deserialization of objects from various formats (JSON, msgpack, etc.).
//! The [`DecodeError`] struct wraps underlying format-specific errors into a
//! unified error type that can be propagated through the application.
//!
//! # Examples
//!
//! ```
//! use object_transfer::errors::DecodeError;
//!
//! // Errors from different formats are automatically converted to DecodeError
//! let json_err: DecodeError<serde_json::Error> = serde_json::from_str::<i32>("invalid").unwrap_err().into();
//! ```

use ::serde::de::Error as DeErr;

use thiserror::Error;

#[derive(Error, Debug)]
#[error("decoding error: {kind}")]
/// Error type for deserialization failures.
///
/// This struct wraps format-agnostic decoding errors from various serialization
/// backends (JSON, msgpack, etc.). It uses `thiserror` to provide consistent
/// error formatting and conversion from format-specific error types.
///
/// The underlying error is boxed to allow different serialization formats to
/// contribute their own error types without requiring a large enum variant.
pub struct DecodeError<E: DeErr + Send + Sync> {
  kind: E,
}

impl<E: DeErr + Send + Sync> DecodeError<E> {
  /// Creates a new `DecodeError` from any error that implements `std::error::Error`.
  pub(crate) fn new(err: E) -> Self {
    DecodeError { kind: err }
  }
}

/// Converts JSON deserialization errors into [`DecodeError`].
///
/// This conversion is only available when the `json` feature is enabled.
#[cfg(feature = "json")]
impl From<serde_json::Error> for DecodeError<serde_json::Error> {
  fn from(err: serde_json::Error) -> Self {
    DecodeError { kind: err }
  }
}

/// Converts MessagePack deserialization errors into [`DecodeError`].
///
/// This conversion is only available when the `msgpack` feature is enabled.
#[cfg(feature = "msgpack")]
impl From<rmp_serde::decode::Error> for DecodeError<rmp_serde::decode::Error> {
  fn from(err: rmp_serde::decode::Error) -> Self {
    DecodeError { kind: err }
  }
}

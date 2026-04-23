//! Serialization and deserialization traits and implementations.
//!
//! This module provides the core abstraction layer for encoding Rust types into bytes
//! and decoding bytes back into Rust types. It is the foundation of the library's
//! "any-format" design, allowing seamless format selection at runtime.
//!
//! # Overview
//!
//! The encoder module defines two primary traits:
//!
//! - [`Encoder`] - Converts Rust types into byte sequences
//! - [`Decoder`] - Converts byte sequences back into Rust types
//!
//! These traits are object-safe and thread-safe, making them suitable for dynamic
//! dispatch through trait objects. This enables applications to support multiple
//! serialization formats without recompiling, or even switch formats at runtime based
//! on configuration or message metadata.
//!
//! # Built-in Formats
//!
//! The library includes ready-to-use implementations for:
//!
//! - **JSON** (feature `json`) - Human-readable, widely compatible, available via [`json`] module
//! - **MessagePack** (feature `msgpack`) - Compact binary format, faster than JSON, available via [`msgpack`] module
//!
//! # Custom Formats
//!
//! You are not limited to built-in formats. Implement [`Encoder`] and [`Decoder`] for any
//! serialization format you need:
//! - Other binary formats: CBOR, Protocol Buffers, Avro, Parquet
//! - Custom formats: domain-specific binary protocols, compressed formats
//! - Specialized formats: for specific use cases or performance requirements
//!
//! # Feature Flags
//!
//! - `json` - Enables JSON encoding/decoding support
//! - `msgpack` - Enables MessagePack encoding/decoding support
//!
//! # Examples
//!
//! Using the JSON encoder/decoder:
//!
//! ```rust,no_run
//! use serde::{Serialize, Deserialize};
//! use object_transfer::encoder::{Encoder, Decoder};
//! #[cfg(feature = "json")]
//! use object_transfer::encoder::{JSONEncoder, JSONDecoder};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Event {
//!     id: u32,
//!     name: String,
//! }
//!
//! # #[cfg(feature = "json")]
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let encoder = JSONEncoder::new();
//! let decoder = JSONDecoder::new();
//!
//! let event = Event { id: 1, name: "test".to_string() };
//! let bytes = encoder.encode(&event)?;
//! let decoded: Event = decoder.decode(bytes)?;
//! # Ok(())
//! # }
//! ```

mod traits;

pub use self::traits::{Decoder, Encoder};

#[cfg(test)]
pub use self::traits::{MockDecoder, MockEncoder};

#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "json")]
pub use self::json::{Decoder as JSONDecoder, Encoder as JSONEncoder};

#[cfg(feature = "msgpack")]
pub mod msgpack;
#[cfg(feature = "msgpack")]
pub use self::msgpack::{
  Decoder as MessagePackDecoder, Encoder as MessagePackEncoder,
};

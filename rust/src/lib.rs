//! This library provides a simple and efficient way to transfer objects between
//! different parts of an application or between different applications through
//! message brokers like NATS.
//! It supports serialization and/or deserialization of various data formats,
//! making it easy to send and/or receive complex data structures.
//!
//! # Pluggable Encoder/Decoder Architecture
//!
//! A key feature of this library is its **"any-format"** design: you can use any
//! serialization format by implementing the [`encoder::Encoder`] and [`encoder::Decoder`] traits.
//! The library doesn't restrict you to built-in formats—JSON, MessagePack, Protocol Buffers,
//! CBOR, or custom formats all work seamlessly.
//!
//! ## Quick Example
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use serde::{Serialize, Deserialize};
//! use object_transfer::{
//!   encoder::{JSONEncoder, JSONDecoder},
//!   Pub, Sub, SubOpt,
//!   traits::PubTrait,
//! };
//!
//! #[derive(Serialize, Deserialize, Clone, Debug)]
//! struct Event {
//!   id: u32,
//!   message: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!   let client = async_nats::connect("demo.nats.io").await?;
//!   let js = Arc::new(async_nats::jetstream::new(client));
//!
//!   // Create a publisher with JSON encoder
//!   let publisher: Pub<Event, _> = Pub::new(
//!     js.clone(),
//!     "events",
//!     Arc::new(JSONEncoder::new()),
//!   );
//!
//!   let event = Event { id: 1, message: "Hello".to_string() };
//!   publisher.publish(&event).await?;
//!   Ok(())
//! }
//! ```
//!
//! ## Implementing Custom Formats
//!
//! Implement [`encoder::Encoder`] and [`encoder::Decoder`] for your format:
//!
//! ```rust
//! use bytes::Bytes;
//! use serde::Serialize;
//! use object_transfer::encoder::Encoder;
//!
//! struct MyFormat;
//! #[derive(serde::Serialize)]
//! struct MyData { id: u32 }
//!
//! impl Encoder for MyFormat {
//!   type Item = MyData;
//!   type Error = std::fmt::Error;
//!
//!   fn encode(&self, item: &Self::Item) -> Result<Bytes, Self::Error> {
//!     Ok(Bytes::from(format!("id:{}", item.id)))
//!   }
//! }
//! ```
//!
//! Then pass your encoder/decoder to [`Pub::new()`] or [`Sub::new()`].
//!
//! # CI/CD Status
//!
//! | Service | Status |
//! |---------|--------|
//! | Crates.io | [![Crates.io Version Img]][Crates.io] |
//! | Code Test | [![Test Rust Code Img]][Test Rust Code] |
//!
//! [Test Rust Code Img]: https://github.com/hiroaki-yamamoto/object-transfer/actions/workflows/test_rust.yml/badge.svg
//! [Test Rust Code]: https://github.com/hiroaki-yamamoto/object-transfer/actions/workflows/test_rust.yml
//! [Crates.io Version Img]: https://img.shields.io/crates/v/object_transfer.svg
//! [Crates.io]: https://crates.io/crates/object_transfer

mod ack_noop;
pub mod brokers;
pub mod encoder;
pub mod errors;
mod options;
mod publisher;
mod subscriber;
pub mod traits;
mod unsub_noop;

#[cfg(test)]
mod tests;

pub use ack_noop::AckNoop;
pub use options::SubOpt;
pub use publisher::Pub;
pub use subscriber::Sub;
pub use traits::{PubTrait, SubTrait, UnSubTrait};
pub use unsub_noop::UnSubNoop;

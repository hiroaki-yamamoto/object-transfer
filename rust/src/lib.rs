//! This library provides a simple and efficient way to transfer objects between
//! different parts of an application or between different applications through
//! message brokers like NATS.
//! It supports serialization and/or deserialization of various data formats,
//! making it easy to send and/or receive complex data structures.
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
pub mod r#enum;
pub mod errors;
#[cfg(feature = "nats")]
pub mod nats;
mod r#pub;
mod sub;
pub mod traits;

#[cfg(test)]
mod tests;

pub use ack_noop::AckNoop;
pub use r#enum::Format;
pub use r#pub::Pub;
pub use sub::Sub;
pub use traits::{PubTrait, SubOptTrait, SubTrait, UnSubTrait};

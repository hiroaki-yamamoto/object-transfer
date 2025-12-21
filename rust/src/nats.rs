//! NATS connector module for pub/sub messaging.
//!
//! This module provides a NATS-based connector,
//! enabling asynchronous message publishing and subscription management.

mod errors;
pub mod impl_ack;
pub mod impl_ctx;
pub mod options;
mod sub_fetcher;

pub use options::AckSubOptions;
pub use sub_fetcher::SubFetcher;

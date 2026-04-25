//! Redis-based implementation for the object-transfer connector.
//!
//! This module provides Redis-backed publisher and subscriber functionality for
//! distributed object transfer. It includes configuration management, error handling,
//! and acknowledgment mechanisms for reliable message delivery.

mod ack;
mod config;
mod errors;
mod group_make;
mod publisher;
mod subscriber;

#[cfg(test)]
mod tests;

pub use self::config::{PublisherConfig, SubscriberConfig};
pub use self::errors::PublishError;
pub use self::publisher::Publisher;
pub use self::subscriber::Subscriber;

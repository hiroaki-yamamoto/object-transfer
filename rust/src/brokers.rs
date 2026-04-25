//! Broker implementations and trait abstractions for message publishing and subscription.
//!
//! This module provides a pluggable architecture for interacting with different message brokers
//! while maintaining a consistent API. It defines core traits that enable broker-agnostic code,
//! along with concrete implementations for popular brokers like NATS and Redis.
//!
//! # Core Traits
//!
//! The module exports two primary traits that form the foundation of the broker abstraction:
//!
//! - [`PubBrokerTrait`]: Provides methods to publish raw byte payloads to a topic.
//! - [`SubBrokerTrait`]: Provides methods to subscribe to a topic and receive a stream
//!   of messages with acknowledgment handles.
//!
//! These traits enable you to write broker-agnostic code and swap implementations at runtime.
//!
//! # Available Implementations
//!
//! - [`nats`]: NATS JetStream broker implementation for high-performance message streaming.
//! - [`redis`]: Redis Streams broker implementation for persistent message queuing.
//!
//! # Error Handling
//!
//! The [`errors`] submodule provides broker-specific and common error types that may be
//! returned by broker operations.
//!
//! # Example
//!
//! ```rust
//! use std::sync::Arc;
//! use object_transfer::brokers::{PubBrokerTrait, SubBrokerTrait};
//!
//! async fn example(
//!     pub_broker: Arc<dyn PubBrokerTrait>,
//!     sub_broker: Arc<dyn SubBrokerTrait>,
//! ) -> Result<(), Box<dyn std::error::Error>> {
//!     // Publish a message
//!     pub_broker.publish("topic", bytes::Bytes::from("message")).await?;
//!
//!     // Subscribe to messages
//!     let mut stream = sub_broker.subscribe().await?;
//!
//!     // Handle incoming messages
//!     while let Some(result) = futures::stream::StreamExt::next(&mut stream).await {
//!         match result {
//!             Ok((payload, ack)) => {
//!                 // Process payload
//!                 ack.ack().await?;
//!             }
//!             Err(e) => eprintln!("Error receiving message: {}", e),
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod errors;
#[cfg(feature = "nats")]
pub mod nats;
#[cfg(feature = "redis")]
pub mod redis;
pub mod traits;

pub use self::traits::{PubBrokerTrait, SubBrokerTrait};

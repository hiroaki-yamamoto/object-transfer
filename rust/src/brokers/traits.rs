//! Traits for broker-agnostic message publishing and subscription.
//!
//! This module defines the core abstractions for interacting with message brokers.
//! It provides trait-based interfaces that enable different broker implementations
//! (e.g., NATS, Redis) to be used interchangeably while maintaining a consistent API.
//!
//! # Overview
//!
//! The module contains two primary traits:
//!
//! - [`PubBrokerTrait`]: Provides functionality to publish raw byte payloads to a broker topic.
//! - [`SubBrokerTrait`]: Provides functionality to subscribe to a broker and receive a stream
//!   of messages with acknowledgment handles.
//!
//! Both traits are designed to work asynchronously and support generic broker implementations.

use ::std::sync::Arc;

use ::async_trait::async_trait;
use ::bytes::Bytes;
use ::futures::stream::BoxStream;
#[cfg(test)]
use ::mockall::automock;

use super::errors::BrokerError;
use crate::traits::AckTrait;

/// Context capable of publishing raw byte payloads.
#[cfg_attr(test, automock)]
#[async_trait]
pub trait PubBrokerTrait {
  /// Publish a raw payload to a subject on the underlying broker.
  ///
  /// # Parameters
  /// - `topic`: Subject or channel name the payload should be delivered to.
  /// - `payload`: Serialized bytes to forward to the transport.
  async fn publish(
    &self,
    topic: &str,
    payload: Bytes,
  ) -> Result<(), BrokerError>;
}

/// Context capable of producing a stream of raw messages with ack handles.
#[async_trait]
pub trait SubBrokerTrait {
  async fn subscribe(
    &self,
  ) -> Result<
    BoxStream<Result<(Bytes, Arc<dyn AckTrait + Send + Sync>), BrokerError>>,
    BrokerError,
  >;
}

#[cfg(test)]
mod tests {
  use ::static_assertions::assert_obj_safe;

  use super::*;

  #[test]
  fn test_pubctx_safety() {
    assert_obj_safe!(PubBrokerTrait);
  }

  #[test]
  fn test_subctx_safety() {
    assert_obj_safe!(SubBrokerTrait);
  }
}

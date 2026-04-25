//! Core trait abstractions for publish-subscribe messaging with strongly-typed items.
//!
//! This module provides a set of traits that define the interfaces for publishing and subscribing
//! to typed messages across different messaging backends (NATS, Redis, etc.). These traits abstract
//! away the complexity of serialization, transport, and acknowledgment handling, allowing you to
//! write backend-agnostic code.
//!
//! # Core Traits
//!
//! - [`PubTrait`]: Publish strongly-typed items that implement [`serde::Serialize`]. Handles encoding
//!   and delivery to the backing message broker.
//! - [`SubTrait`]: Subscribe to a stream of strongly-typed items that implement [`serde::de::DeserializeOwned`].
//!   Returns a stream of decoded messages paired with acknowledgment handles.
//! - [`AckTrait`]: Acknowledge receipt of a message after it has been successfully processed.
//! - [`UnSubTrait`]: Cancel an active subscription gracefully.
//!
//! # Dispatch Patterns
//!
//! The library supports two primary usage patterns for different performance and flexibility trade-offs:
//!
//! ## Static Dispatch (Recommended)
//!
//! Use generic trait bounds to maintain full type information at compile time. This enables
//! monomorphization, inlining, and zero runtime overhead. Ideal for performance-critical paths.
//!
//! ### Publishing with Static Dispatch
//!
//! ```rust
//! use object_transfer::traits::PubTrait;
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Event {
//!     id: u32,
//!     message: String,
//! }
//!
//! async fn send_event<P: PubTrait<Item = Event>>(publisher: &P, event: &Event) -> Result<(), Box<dyn std::error::Error>> {
//!     publisher.publish(event).await?;
//!     Ok(())
//! }
//! ```
//!
//! ### Subscribing with Static Dispatch
//!
//! ```rust
//! use object_transfer::traits::{SubTrait, AckTrait};
//! use serde::Deserialize;
//! use futures::stream::StreamExt;
//!
//! #[derive(Deserialize)]
//! struct Event {
//!     id: u32,
//!     message: String,
//! }
//!
//! async fn receive_events<S: SubTrait<Item = Event>>(
//!     subscriber: &S,
//! ) -> Result<(), Box<dyn std::error::Error>> {
//!     let mut stream = subscriber.subscribe().await?;
//!     while let Some(result) = stream.next().await {
//!         match result {
//!             Ok((event, ack)) => {
//!                 # println!("Received: {:?}", event);
//!                 ack.ack().await.ok();
//!             }
//!             Err(e) => eprintln!("Error: {:?}", e),
//!         }
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ### Relay Pattern with Generic Implementations
//!
//! A common pattern is to relay messages between publishers and subscribers:
//!
//! ```rust
//! use object_transfer::traits::{PubTrait, SubTrait};
//! use serde::{Serialize, Deserialize};
//! use futures::stream::StreamExt;
//!
//! #[derive(Serialize, Deserialize, Clone)]
//! struct Message {
//!     id: u64,
//!     content: String,
//! }
//!
//! async fn relay<P, S>(publisher: &P, subscriber: &S) -> Result<(), Box<dyn std::error::Error>>
//! where
//!     P: PubTrait<Item = Message>,
//!     S: SubTrait<Item = Message>,
//! {
//!     let mut stream = subscriber.subscribe().await?;
//!     while let Some(result) = stream.next().await {
//!         if let Ok((msg, ack)) = result {
//!             publisher.publish(&msg).await.ok();
//!             ack.ack().await.ok();
//!         }
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Dynamic Dispatch (Trait Objects)
//!
//! Use trait objects for runtime polymorphism when the concrete type is unknown or must be
//! determined at runtime. This introduces a small runtime cost but provides maximum flexibility.
//! This pattern is common in plugin systems or when accepting multiple publisher/subscriber implementations.
//!
//! ### Publishing with Dynamic Dispatch
//!
//! ```rust
//! use std::sync::Arc;
//! use object_transfer::traits::PubTrait;
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Event {
//!     id: u32,
//!     data: String,
//! }
//!
//! async fn send_via_any_publisher(
//!     publisher: Arc<dyn PubTrait<Item = Event, EncodeErr = serde_json::Error>>,
//!     event: &Event,
//! ) -> Result<(), Box<dyn std::error::Error>> {
//!     publisher.publish(event).await?;
//!     Ok(())
//! }
//! ```
//!
//! ### Subscribing with Dynamic Dispatch
//!
//! ```rust
//! use std::sync::Arc;
//! use object_transfer::traits::{SubTrait, AckTrait};
//! use serde::Deserialize;
//! use futures::stream::StreamExt;
//!
//! #[derive(Deserialize)]
//! struct Event {
//!     id: u32,
//!     data: String,
//! }
//!
//! async fn receive_via_any_subscriber(
//!     subscriber: Arc<dyn SubTrait<Item = Event, DecodeErr = serde_json::Error>>,
//! ) -> Result<(), Box<dyn std::error::Error>> {
//!     let mut stream = subscriber.subscribe().await?;
//!     while let Some(result) = stream.next().await {
//!         match result {
//!             Ok((event, ack)) => {
//!                 # println!("Received: {:?}", event);
//!                 ack.ack().await.ok();
//!             }
//!             Err(e) => eprintln!("Error: {:?}", e),
//!         }
//!     }
//!     Ok(())
//! }
//! ```
//!
//! # Integration with Concrete Backends
//!
//! Concrete implementations of these traits are provided for different backends:
//! - NATS: See the [`crate::nats`] module (requires `nats` feature)
//! - Redis: See the [`crate::redis`] module (requires `redis` feature)
//!
//! # Error Handling
//!
//! Operations may fail for various reasons (encoding errors, network issues, etc.).
//! Each trait method returns a `Result` with a specific error type:
//!
//! - [`PubTrait::publish()`] returns [`crate::errors::PubError<Self::EncodeErr>`]
//! - [`SubTrait::subscribe()`] returns [`crate::errors::SubError<Self::DecodeErr>`]
//! - [`AckTrait::ack()`] returns [`crate::errors::AckError`]
//! - [`UnSubTrait::unsubscribe()`] returns [`crate::errors::UnSubError`]
//!

use ::std::sync::Arc;

use ::async_trait::async_trait;
use ::futures::stream::BoxStream;
use ::serde::{
  de::{DeserializeOwned, Error as DeErr},
  ser::{Error as EncErr, Serialize},
};

use crate::errors::{AckError, PubError, SubError, UnSubError};

#[cfg(test)]
use crate::tests::{entity::TestEntity, error::MockDeErr, error::MockEncErr};
#[cfg(test)]
use ::mockall::automock;

/// Abstraction for publishing typed items.
///
/// Implementors handle serialization and delivery to a concrete backend.
#[cfg_attr(test, automock(type Item = TestEntity; type EncodeErr = MockEncErr;))]
#[async_trait]
pub trait PubTrait {
  type Item: Serialize + Send + Sync;
  type EncodeErr: EncErr + Send + Sync;
  /// Publish a serializable item through the implementor.
  ///
  /// # Parameters
  /// - `obj`: The typed item to serialize and send to the backing transport.
  async fn publish(
    &self,
    obj: &Self::Item,
  ) -> Result<(), PubError<Self::EncodeErr>>;
}

/// Acknowledge receipt of a message.
#[cfg_attr(test, automock)]
#[async_trait]
pub trait AckTrait {
  async fn ack(&self) -> Result<(), AckError>;
}

/// Subscription interface returning a stream of decoded items and ack handles.
#[async_trait]
pub trait SubTrait {
  type Item: DeserializeOwned + Send + Sync;
  type DecodeErr: DeErr + Send + Sync;
  async fn subscribe(
    &self,
  ) -> Result<
    BoxStream<
      Result<
        (Self::Item, Arc<dyn AckTrait + Send + Sync>),
        SubError<Self::DecodeErr>,
      >,
    >,
    SubError<Self::DecodeErr>,
  >;
}

/// Allows canceling a subscription.
#[cfg_attr(test, automock)]
#[async_trait]
pub trait UnSubTrait {
  async fn unsubscribe(&self) -> Result<(), UnSubError>;
}

#[cfg(test)]
mod test {
  use ::static_assertions::assert_obj_safe;

  use super::*;
  #[test]
  fn test_pub_safety() {
    assert_obj_safe!(PubTrait<Item = TestEntity, EncodeErr = MockEncErr>);
  }

  #[test]
  fn test_ack_safety() {
    assert_obj_safe!(AckTrait);
  }

  #[test]
  fn test_sub_safety() {
    assert_obj_safe!(SubTrait<Item = TestEntity, DecodeErr = MockDeErr  >);
  }

  #[test]
  fn test_unsub_safety() {
    assert_obj_safe!(UnSubTrait);
  }
}

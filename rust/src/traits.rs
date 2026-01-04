//! Core trait abstractions for pub/sub messaging with typed and untyped interfaces.
//!
//! This module provides a set of traits that abstract over different messaging
//! backends and transport layers. The traits are organized into two main
//! categories:
//!
//! ## Typed Traits
//!
//! These traits work with strongly-typed items:
//! - [`PubTrait`]: Publish strongly-typed items that implement [`serde::Serialize`].
//! - [`SubTrait`]: Subscribe to a stream of strongly-typed items that implement
//!   [`serde::de::DeserializeOwned`].
//! - [`AckTrait`]: Acknowledge receipt of a message.
//! - [`UnSubTrait`]: Cancel a subscription.
//!
//! ## Untyped (Context) Traits
//!
//! These traits work with raw byte payloads and topics:
//! - [`PubCtxTrait`]: Publish raw byte payloads to specific topics.
//! - [`SubCtxTrait`]: Subscribe to a stream of raw byte messages from a topic.
//!
//! ## Additional Traits
//!
//! - [`SubOptTrait`]: Configure subscription options like auto-acknowledgment and format.
//!
//! # Dynamic Dispatch Usage
//!
//! These traits are designed to work seamlessly with dynamic dispatch (trait objects),
//! enabling flexible, runtime polymorphism. Here are common patterns:
//!
//! ## Publishing with Dynamic Dispatch
//!
//! ```rust
//! use std::sync::Arc;
//! use object_transfer::traits::PubTrait;
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct MyMessage {
//!     data: String,
//! }
//!
//! async fn send_message(publisher: Arc<dyn PubTrait<Item = MyMessage>>) {
//!     let msg = MyMessage { data: "hello".to_string() };
//!     publisher.publish(&msg).await.ok();
//! }
//! ```
//!
//! ## Subscribing with Dynamic Dispatch
//!
//! ```rust
//! use std::sync::Arc;
//! use object_transfer::traits::{SubTrait, AckTrait};
//! use serde::Deserialize;
//! use futures::stream::StreamExt;
//!
//! #[derive(Deserialize)]
//! struct MyMessage {
//!     data: String,
//! }
//!
//! async fn receive_messages(
//!     subscriber: Arc<dyn SubTrait<Item = MyMessage>>
//! ) {
//!     if let Ok(mut stream) = subscriber.subscribe().await {
//!         while let Some(Ok((msg, ack))) = stream.next().await {
//!             // Process the message
//!             let _ = ack.ack().await;
//!         }
//!     }
//! }
//! ```
//!
//! ## Working with Raw Payloads
//!
//! For scenarios requiring lower-level control, use context traits:
//!
//! ```rust
//! use std::sync::Arc;
//! use object_transfer::traits::PubCtxTrait;
//! use bytes::Bytes;
//!
//! async fn send_raw(ctx: Arc<dyn PubCtxTrait>) {
//!     let payload = Bytes::from("raw data");
//!     ctx.publish("topic/name", payload).await.ok();
//! }
//! ```
//!
//! # Static Dispatch Usage
//!
//! For maximum performance and compile-time guarantees, use static dispatch with
//! generic trait bounds. This approach leverages monomorphization to eliminate
//! runtime overhead and enable inlining.
//!
//! ## Publishing with Static Dispatch
//!
//! ```rust
//! use object_transfer::traits::PubTrait;
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct MyMessage {
//!     data: String,
//! }
//!
//! async fn send_message<P: PubTrait<Item = MyMessage>>(publisher: &P) {
//!     let msg = MyMessage { data: "hello".to_string() };
//!     publisher.publish(&msg).await.ok();
//! }
//! ```
//!
//! ## Subscribing with Static Dispatch
//!
//! ```rust
//! use object_transfer::traits::{SubTrait, AckTrait};
//! use serde::Deserialize;
//! use futures::stream::StreamExt;
//!
//! #[derive(Deserialize)]
//! struct MyMessage {
//!     data: String,
//! }
//!
//! async fn receive_messages<S: SubTrait<Item = MyMessage>>(subscriber: &S) {
//!     if let Ok(mut stream) = subscriber.subscribe().await {
//!         while let Some(Ok((msg, ack))) = stream.next().await {
//!             // Process the message
//!             let _ = ack.ack().await;
//!         }
//!     }
//! }
//! ```
//!
//! ## Generic Over Multiple Trait Implementations
//!
//! Static dispatch excels when working with multiple trait implementations:
//!
//! ```rust
//! use object_transfer::traits::{PubTrait, SubTrait, AckTrait};
//! use serde::{Serialize, Deserialize};
//! use futures::stream::StreamExt;
//!
//! #[derive(Serialize, Deserialize)]
//! struct Event {
//!     id: u64,
//! }
//!
//! async fn relay_events<P, S>(publisher: &P, subscriber: &S)
//! where
//!     P: PubTrait<Item = Event>,
//!     S: SubTrait<Item = Event>,
//! {
//!     if let Ok(mut stream) = subscriber.subscribe().await {
//!         while let Some(Ok((event, ack))) = stream.next().await {
//!             publisher.publish(&event).await.ok();
//!             let _ = ack.ack().await;
//!         }
//!     }
//! }
//! ```
//!

use ::bytes::Bytes;
use ::std::sync::Arc;

use ::async_trait::async_trait;
use ::futures::stream::BoxStream;
use ::serde::{Serialize, de::DeserializeOwned};

use crate::r#enum::Format;
use crate::errors::{AckError, PubError, SubError, UnSubError};

#[cfg(test)]
use crate::tests::entity::TestEntity;
#[cfg(test)]
use ::mockall::automock;

/// Abstraction for publishing typed items.
///
/// Implementors handle serialization and delivery to a concrete backend.
#[cfg_attr(test, automock(type Item = TestEntity;))]
#[async_trait]
pub trait PubTrait {
  type Item: Serialize + Send + Sync;
  /// Publish a serializable item through the implementor.
  ///
  /// # Parameters
  /// - `obj`: The typed item to serialize and send to the backing transport.
  async fn publish(&self, obj: &Self::Item) -> Result<(), PubError>;
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
  async fn subscribe(
    &self,
  ) -> Result<
    BoxStream<Result<(Self::Item, Arc<dyn AckTrait + Send + Sync>), SubError>>,
    SubError,
  >;
}

/// Allows canceling a subscription.
#[cfg_attr(test, automock)]
#[async_trait]
pub trait UnSubTrait {
  async fn unsubscribe(&self) -> Result<(), UnSubError>;
}

/// Context capable of publishing raw byte payloads.
#[cfg_attr(test, automock)]
#[async_trait]
pub trait PubCtxTrait {
  /// Publish a raw payload to a subject on the underlying broker.
  ///
  /// # Parameters
  /// - `topic`: Subject or channel name the payload should be delivered to.
  /// - `payload`: Serialized bytes to forward to the transport.
  async fn publish(&self, topic: &str, payload: Bytes)
  -> Result<(), PubError>;
}

/// Context capable of producing a stream of raw messages with ack handles.
#[async_trait]
pub trait SubCtxTrait {
  async fn subscribe(
    &self,
  ) -> Result<
    BoxStream<Result<(Bytes, Arc<dyn AckTrait + Send + Sync>), SubError>>,
    SubError,
  >;
}

/// Options that influence subscription behavior such as auto-ack and format.
#[cfg_attr(test, automock)]
pub trait SubOptTrait {
  fn get_auto_ack(&self) -> bool;
  fn get_format(&self) -> Format;
}

#[cfg(test)]
mod test {
  use ::static_assertions::assert_obj_safe;

  use super::*;
  #[test]
  fn test_pub_safety() {
    assert_obj_safe!(PubTrait<Item = TestEntity>);
  }

  #[test]
  fn test_ack_safety() {
    assert_obj_safe!(AckTrait);
  }

  #[test]
  fn test_sub_safety() {
    assert_obj_safe!(SubTrait<Item = TestEntity>);
  }

  #[test]
  fn test_unsub_safety() {
    assert_obj_safe!(UnSubTrait);
  }

  #[test]
  fn test_pubctx_safety() {
    assert_obj_safe!(PubCtxTrait);
  }

  #[test]
  fn test_subctx_safety() {
    assert_obj_safe!(SubCtxTrait);
  }

  #[test]
  fn test_subopt_safety() {
    assert_obj_safe!(SubOptTrait);
  }
}

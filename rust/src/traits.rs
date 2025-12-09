use ::bytes::Bytes;
use ::std::sync::Arc;

use ::async_trait::async_trait;
use ::futures::stream::BoxStream;
use ::serde::{Serialize, de::DeserializeOwned};

use crate::r#enum::Format;
use crate::error::Error;

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
  async fn publish(&self, obj: &Self::Item) -> Result<(), Error>;
}

/// Acknowledge receipt of a message.
#[cfg_attr(test, automock)]
#[async_trait]
pub trait AckTrait {
  async fn ack(&self) -> Result<(), Error>;
}

/// Subscription interface returning a stream of decoded items and ack handles.
#[async_trait]
pub trait SubTrait {
  type Item: DeserializeOwned + Send + Sync;
  async fn subscribe(
    &self,
  ) -> Result<
    BoxStream<Result<(Self::Item, Arc<dyn AckTrait + Send + Sync>), Error>>,
    Error,
  >;
}

/// Allows canceling a subscription.
#[cfg_attr(test, automock)]
#[async_trait]
pub trait UnSubTrait {
  async fn unsubscribe(&self) -> Result<(), Error>;
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
  async fn publish(&self, topic: &str, payload: Bytes) -> Result<(), Error>;
}

/// Context capable of producing a stream of raw messages with ack handles.
#[async_trait]
pub trait SubCtxTrait {
  async fn subscribe(
    &self,
  ) -> Result<
    BoxStream<Result<(Bytes, Arc<dyn AckTrait + Send + Sync>), Error>>,
    Error,
  >;
}

/// Options that influence subscription behavior such as auto-ack and format.
#[cfg_attr(test, automock)]
pub trait SubOptTrait {
  fn get_auto_ack(&self) -> bool;
  fn get_format(&self) -> Format;
}

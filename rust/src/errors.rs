//! Error definitions shared across the crate.
//! Defines high-level error types (AckError, PubError, SubError, UnSubError)
//! that use BrokerError as a common wrapper for NATS, JetStream, and
//! serialization errors.

mod ack;
mod decode;
mod encode;
mod r#pub;
mod sub;
mod unsub;

pub use self::ack::AckError;
pub use self::decode::DecodeError;
pub use self::encode::EncodeError;
pub use self::r#pub::PubError;
pub use self::sub::SubError;
pub use self::unsub::UnSubError;
pub use crate::brokers::errors::BrokerError;

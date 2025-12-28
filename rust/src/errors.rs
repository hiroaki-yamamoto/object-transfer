//! Error definitions shared across the crate.
//! Wraps NATS, JetStream, and serialization errors under a single type.

mod ack;
mod broker;
mod r#pub;
mod sub;
mod unsub;

pub use self::ack::AckError;
pub use self::broker::BrokerError;
pub use self::r#pub::PubError;
pub use self::sub::SubError;
pub use self::unsub::UnSubError;

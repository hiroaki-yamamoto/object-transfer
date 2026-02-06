use ::redis::aio::MultiplexedConnection;

use super::config::SubscriberConfig;

#[derive(Clone)]
pub struct Subscriber {
  con: MultiplexedConnection,
  cfg: SubscriberConfig,
}

use ::std::sync::Arc;

use ::async_stream::try_stream;
use ::async_trait::async_trait;
use ::bytes::Bytes;
use ::redis::AsyncCommands;
use ::redis::aio::MultiplexedConnection;

use crate::errors::SubError;
use crate::traits::{AckTrait, SubCtxTrait};

use super::config::SubscriberConfig;

#[derive(Clone)]
pub struct Subscriber {
  con: MultiplexedConnection,
  cfg: SubscriberConfig,
}

impl Subscriber {
  pub fn new(con: &MultiplexedConnection, cfg: SubscriberConfig) -> Self {
    Self {
      con: con.clone(),
      cfg,
    }
  }
}

#[async_trait]
impl SubCtxTrait for Subscriber {
  async fn subscribe(
    &self,
  ) -> Result<
    BoxStream<Result<(Bytes, Arc<dyn AckTrait + Send + Sync>), SubError>>,
    SubError,
  > {
  }
}

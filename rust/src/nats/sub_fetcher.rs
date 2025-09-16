use ::std::sync::Arc;

use ::async_nats::jetstream::context::Context;

use super::options::AckSubOptions;
use crate::traits::SubCtxTrait;

#[derive(Debug)]
pub(super) struct SubFetcher {
  ctx: Context,
  options: Arc<AckSubOptions>,
}

impl SubFetcher {
  pub fn new(ctx: Context, options: Arc<AckSubOptions>) -> Self {
    Self { ctx, options }
  }
}

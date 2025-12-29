use ::std::error::Error as StdError;

use ::thiserror::Error;

#[derive(Error, Debug)]
#[error(transparent)]
pub struct BrokerError(#[from] Box<dyn StdError + Send + Sync>);

impl BrokerError {
  pub fn new<E>(err: E) -> Self
  where
    E: StdError + Send + Sync + 'static,
  {
    Self(Box::new(err))
  }
}

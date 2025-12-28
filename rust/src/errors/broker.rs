use ::std::error::Error as StdError;

use ::thiserror::Error;

#[derive(Error, Debug)]
#[error(transparent)]
pub struct BrokerError(#[from] pub Box<dyn StdError + Send + Sync>);

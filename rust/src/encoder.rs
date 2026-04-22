mod traits;

pub use self::traits::{Decoder, Encoder};

#[cfg(test)]
pub use self::traits::{MockDecoder, MockEncoder};

#[cfg(feature = "json")]
mod json;
#[cfg(feature = "json")]
pub use self::json::{JSONDecoder, JSONEncoder};

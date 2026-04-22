mod traits;

pub use self::traits::{Decoder, Encoder};

#[cfg(test)]
pub use self::traits::{MockDecoder, MockEncoder};

#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "json")]
pub use self::json::{Decoder as JSONDecoder, Encoder as JSONEncoder};

#[cfg(feature = "msgpack")]
pub mod msgpack;
#[cfg(feature = "msgpack")]
pub use self::msgpack::{
  Decoder as MessagePackDecoder, Encoder as MessagePackEncoder,
};

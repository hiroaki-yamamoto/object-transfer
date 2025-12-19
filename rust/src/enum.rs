//! Enumerations shared across the crate.
//! Currently only exposes the `Format` enum representing supported
//! serialization/deserialization formats.

#[cfg(test)]
use ::std::string::ToString;

/// Supported message serialization/deserialization formats.
/// Note that the structure to be serialized / deserialized must implement
/// [`serde::Serialize`] and [`serde::Deserialize`].
#[derive(Debug, Clone, Copy)]
pub enum Format {
  /// MessagePack serialization format.
  MessagePack,
  /// JSON serialization format.
  JSON,
}

#[cfg(test)]
impl ToString for Format {
  fn to_string(&self) -> String {
    return match self {
      Format::MessagePack => "MessagePack",
      Format::JSON => "JSON",
    }
    .to_string();
  }
}

#[cfg(test)]
use ::std::string::ToString;

#[derive(Debug, Clone, Copy)]
pub enum Format {
  MessagePack,
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

use serde::{de::Error as DeErr, ser::Error as EncErr};

use thiserror::Error;

#[derive(Error, Debug)]
#[error("Mock encoding error")]
pub struct MockEncErr;

impl EncErr for MockEncErr {
  fn custom<T: std::fmt::Display>(_msg: T) -> Self {
    Self
  }
}

#[derive(Error, Debug)]
#[error("Mock decoding error")]
pub struct MockDeErr;

impl DeErr for MockDeErr {
  fn custom<T: std::fmt::Display>(_msg: T) -> Self {
    Self
  }
}

#[derive(Error, Debug)]
#[error("Broker Test Error Entity")]
pub struct MockBrokerErr;

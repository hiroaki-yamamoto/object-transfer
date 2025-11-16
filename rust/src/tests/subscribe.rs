use futures::stream::{Iter, iter};

use super::entity::TestEntity;

pub struct SubscribeMock {
  stream: Iter<std::vec::IntoIter<TestEntity>>,
}

impl SubscribeMock {
  pub fn new(data: Vec<super::entity::TestEntity>) -> Self {
    Self { stream: iter(data) }
  }
}

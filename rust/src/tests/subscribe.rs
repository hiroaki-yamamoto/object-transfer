use super::stream::StreamMock;

pub struct SubscribeMock {
  stream: StreamMock,
}

impl SubscribeMock {
  pub fn new(data: Vec<super::entity::TestEntity>) -> Self {
    Self {
      stream: StreamMock::new(data),
    }
  }
}

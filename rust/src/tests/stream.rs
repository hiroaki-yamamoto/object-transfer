use ::futures::stream::Stream;

use super::entity::TestEntity;

pub struct StreamMock {
  pub data: Vec<TestEntity>,
  cur: usize,
}

impl StreamMock {
  pub fn new(data: Vec<TestEntity>) -> Self {
    Self { data, cur: 0 }
  }

  pub fn reset(&mut self) {
    self.cur = 0;
  }
}

impl Stream for StreamMock {
  type Item = TestEntity;

  fn poll_next(
    mut self: std::pin::Pin<&mut Self>,
    _cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Option<Self::Item>> {
    if self.cur >= self.data.len() {
      return std::task::Poll::Ready(None);
    }
    let item = self.data[self.cur].clone();
    self.cur += 1;
    std::task::Poll::Ready(Some(item))
  }
}

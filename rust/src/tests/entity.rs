#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct TestEntity {
  pub id: u32,
  pub name: String,
}

impl TestEntity {
  pub fn new(id: u32, name: &str) -> Self {
    Self {
      id,
      name: name.to_string(),
    }
  }
}

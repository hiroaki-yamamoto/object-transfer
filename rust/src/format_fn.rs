use ::std::error::Error;

type GenericError = Box<dyn Error + Send + Sync>;
pub type ToVecFn<T> =
  Box<dyn Fn(&T) -> Result<Vec<u8>, GenericError> + Send + Sync>;
pub type FromSliceFn<'a, T> =
  Box<dyn Fn(&'a [u8]) -> Result<T, GenericError> + Send + Sync>;

pub fn to_vec_fn<T, F, E>(func: F) -> ToVecFn<T>
where
  F: Fn(&T) -> Result<Vec<u8>, E> + Send + Sync + 'static,
  E: Error + Send + Sync + 'static,
{
  Box::new(move |data| func(data).map_err(|e| Box::new(e) as GenericError))
}

pub fn from_slice_fn<'a, T, F, E>(func: F) -> FromSliceFn<'a, T>
where
  F: Fn(&'a [u8]) -> Result<T, E> + Send + Sync + 'static,
  E: Error + Send + Sync + 'static,
{
  Box::new(move |data| func(data).map_err(|e| Box::new(e) as GenericError))
}

#[cfg(test)]
mod test {
  use ::rmp_serde::{from_slice as from_msgpack, to_vec as to_msgpack};
  use ::serde_json::{from_slice as from_json, to_vec as to_json};

  use super::*;
  use crate::tests::entity::TestEntity;

  #[test]
  fn test_format_to_vec_fn_generalization() {
    let _: ToVecFn<TestEntity> = to_vec_fn(to_json);
    let _: ToVecFn<TestEntity> = to_vec_fn(to_msgpack);
  }

  #[test]
  fn test_format_from_slice_fn_generalization() {
    let _: FromSliceFn<TestEntity> = from_slice_fn(from_json);
    let _: FromSliceFn<TestEntity> = from_slice_fn(from_msgpack);
  }
}

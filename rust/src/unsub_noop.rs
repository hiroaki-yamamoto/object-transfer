//! A no-operation unsubscribe implementation.
//!
//! This module provides a no-op implementation of the unsubscribe trait,
//! useful for scenarios where unsubscribe operations are not needed or
//! should be skipped without performing any actual work.

use async_trait::async_trait;

use crate::errors::UnSubError;
use crate::traits::UnSubTrait;

/// A no-operation unsubscribe handler.
///
/// `UnSubNoop` is a simple implementation of [`UnSubTrait`](crate::traits::UnSubTrait)
/// that performs no operations when unsubscribe is called. It always returns `Ok(())`
/// without taking any action.
///
/// This is useful for:
/// - Default implementations where unsubscribe is not required
/// - Testing and mocking scenarios
/// - Cases where subscription cleanup is not necessary
pub struct UnSubNoop;

#[async_trait]
impl UnSubTrait for UnSubNoop {
  /// Performs a no-operation unsubscribe.
  ///
  /// This method always completes successfully without performing any work.
  ///
  /// # Returns
  ///
  /// Always returns `Ok(())`.
  async fn unsubscribe(&self) -> Result<(), UnSubError> {
    Ok(())
  }
}

use ::redis::AsyncTypedCommands;
use ::redis::RedisResult;

/// Creates a consumer group for a Redis stream, ensuring idempotent behavior.
///
/// If the group already exists, the function returns `Ok(())` instead of failing.
/// This is achieved by ignoring "BUSYGROUP" errors, which are returned when attempting
/// to create a group that already exists.
///
/// # Arguments
///
/// * `con` - An async-capable Redis command connection
/// * `stream_name` - The name of the Redis stream
/// * `group_name` - The name of the consumer group to create
///
/// # Returns
///
/// * `Ok(())` - If the group was created or already exists
/// * `Err(RedisError)` - If the operation fails for any reason other than group already existing
pub(super) async fn make_stream_group(
  mut con: impl AsyncTypedCommands,
  stream_name: impl Into<String>,
  group_name: impl Into<String>,
) -> RedisResult<()> {
  match con
    .xgroup_create_mkstream(stream_name.into(), group_name.into(), "$")
    .await
  {
    Ok(_) => Ok(()),
    Err(err) => {
      // Ignore "BUSYGROUP" errors (group already exists) to make subscription idempotent.
      if err.code() == Some("BUSYGROUP") {
        Ok(())
      } else {
        Err(err)
      }
    }
  }
}

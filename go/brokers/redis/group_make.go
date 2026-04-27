package redis

import (
	"context"
	"strings"

	"github.com/redis/go-redis/v9"
)

// MakeStreamGroup creates a consumer group for a Redis stream, ensuring idempotent behavior.
//
// If the group already exists, the function returns nil instead of failing.
// This is achieved by ignoring "BUSYGROUP" errors, which are returned when attempting
// to create a group that already exists.
//
// # Arguments
//
//   - ctx: A context for cancellation and timeouts
//   - cmd: A Redis command interface (typically *redis.Client or redis.Cmdable)
//   - streamName: The name of the Redis stream
//   - groupName: The name of the consumer group to create
//
// # Returns
//
//   - nil - If the group was created or already exists
//   - error - If the operation fails for any reason other than group already existing
func MakeStreamGroup(
	ctx context.Context,
	cmd redis.Cmdable,
	streamName, groupName string,
) error {
	err := cmd.XGroupCreateMkStream(ctx, streamName, groupName, "$").Err()
	if err != nil {
		// Ignore "BUSYGROUP" errors (group already exists) to make subscription idempotent.
		if strings.Contains(err.Error(), "BUSYGROUP") {
			return nil
		}
		return err
	}
	return nil
}

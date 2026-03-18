package redis

import (
	"context"

	"github.com/redis/go-redis/v9"

	"github.com/hiroaki-yamamoto/object-transfer/go/errors"
)

// Ack represents an acknowledgment for a message in a Redis stream consumer group.
//
// The Ack struct is responsible for acknowledging a message that has been
// successfully processed by a consumer in a Redis stream. It maintains a connection
// to the Redis instance and stores the necessary identifiers to track which message
// should be acknowledged.
type Ack struct {
	client     redis.Cmdable
	group      string
	streamName string
	id         string
}

// NewAck creates a new Ack instance for acknowledging a Redis stream message.
//
// # Arguments
//
//   - client: A Redis command interface (typically *redis.Client or redis.Cmdable)
//   - group: The consumer group name
//   - streamName: The name of the Redis stream
//   - id: The unique identifier of the message to acknowledge
//
// # Returns
//
// A new Ack instance configured with the provided parameters.
func NewAck(
	client redis.Cmdable,
	group, streamName, id string,
) *Ack {
	return &Ack{
		client:     client,
		group:      group,
		streamName: streamName,
		id:         id,
	}
}

// Ack acknowledges the message in the Redis stream consumer group.
//
// This method marks the message with the stored ID as processed within the
// consumer group. Once acknowledged, the message will no longer be pending
// for the consumer group.
//
// # Arguments
//
//   - ctx: Context for cancellation and timeouts
//
// # Returns
//
// An error if the acknowledgment operation failed (e.g., connection error).
func (a *Ack) Ack(ctx context.Context) error {
	err := a.client.XAck(ctx, a.streamName, a.group, a.id).Err()
	if err != nil {
		return errors.AckBrokerError(
			errors.NewBrokerError(err),
		)
	}
	return nil
}

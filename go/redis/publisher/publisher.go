package publisher

import (
	"context"

	goredis "github.com/redis/go-redis/v9"

	"github.com/hiroaki-yamamoto/object-transfer/go/errors"
	bredis "github.com/hiroaki-yamamoto/object-transfer/go/redis"
	"github.com/hiroaki-yamamoto/object-transfer/go/redis/config"
	rediserrors "github.com/hiroaki-yamamoto/object-transfer/go/redis/errors"
)

// Publisher is a Redis-based message publisher that sends messages to Redis streams.
//
// The Publisher struct provides functionality to publish messages to Redis streams
// with support for consumer groups and stream size management. It implements the
// IPubBroker interface to provide a consistent interface for publishing operations.
type Publisher struct {
	// client is the Redis client used for publishing messages.
	client goredis.Cmdable
	// cfg holds the configuration settings for the publisher, including group name and stream length.
	cfg *config.PublisherConfig
}

// New creates a new Publisher instance.
//
// # Arguments
//
//   - client: A Redis command interface (typically *goredis.Client or goredis.Cmdable)
//   - cfg: Publisher configuration settings
//
// # Returns
//
// A new Publisher instance configured with the provided client and settings.
func New(client goredis.Cmdable, cfg *config.PublisherConfig) *Publisher {
	return &Publisher{
		client: client,
		cfg:    cfg,
	}
}

// Publish sends a raw payload to a topic on the Redis broker.
//
// The method:
// 1. Determines the consumer group name (uses config value or defaults to topic name)
// 2. Creates or ensures the stream consumer group exists
// 3. Adds the message to the Redis stream with MAXLEN constraint
//
// # Arguments
//
//   - ctx: Context for cancellation and timeouts
//   - topic: The Redis stream name (subject/channel) to publish to
//   - payload: The serialized bytes to publish
//
// # Returns
//
// An errors.PubError if the operation fails.
func (p *Publisher) Publish(ctx context.Context, topic string, payload []byte) *errors.PubError {
	groupName := topic
	if p.cfg.GroupName != nil {
		groupName = *p.cfg.GroupName
	}

	// Ensure the stream consumer group exists
	err := bredis.MakeStreamGroup(ctx, p.client, topic, groupName)
	if err != nil {
		return errors.PubBrokerError(
			errors.NewBrokerError(
				rediserrors.NewGroupCreationError(err),
			),
		)
	}

	// Add message to the stream with MAXLEN constraint
	err = p.client.XAdd(ctx, &goredis.XAddArgs{
		Stream: topic,
		MaxLen: int64(p.cfg.StreamLength),
		Approx: true,
		Values: []interface{}{"data", payload},
	}).Err()
	if err != nil {
		return errors.PubBrokerError(
			errors.NewBrokerError(
				rediserrors.NewPushError(err),
			),
		)
	}

	return nil
}

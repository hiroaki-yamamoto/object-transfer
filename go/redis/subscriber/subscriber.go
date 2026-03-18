package subscriber

import (
	"context"
	"time"

	"github.com/redis/go-redis/v9"

	"github.com/hiroaki-yamamoto/object-transfer/go/errors"
	bredis "github.com/hiroaki-yamamoto/object-transfer/go/redis"
	"github.com/hiroaki-yamamoto/object-transfer/go/redis/config"
	rediserrors "github.com/hiroaki-yamamoto/object-transfer/go/redis/errors"

	"github.com/hiroaki-yamamoto/object-transfer/go/interfaces"
)

// Subscriber is a Redis-based message subscriber that handles subscription to Redis streams.
//
// This struct manages subscription to Redis stream topics using consumer groups,
// allowing for acknowledgment of processed messages and reliable message delivery.
type Subscriber struct {
	client redis.Cmdable
	cfg    *config.SubscriberConfig
}

// New creates a new subscriber instance.
//
// # Arguments
//
//   - client: A Redis command interface (typically *redis.Client or redis.Cmdable)
//   - cfg: The subscriber configuration containing topic, group, and consumer names
//
// # Returns
//
// A new Subscriber instance configured with the provided client and settings.
func New(client redis.Cmdable, cfg *config.SubscriberConfig) *Subscriber {
	return &Subscriber{
		client: client,
		cfg:    cfg,
	}
}

// handleStreamIds processes stream IDs and extracts payloads with their acknowledgment handlers.
//
// # Arguments
//
//   - _ (ctx): Context for cancellation
//   - streamIDs: The stream IDs to process
//
// # Returns
//
// A slice of tuples containing (payload bytes, IAck handler)
func (s *Subscriber) handleStreamIds(
	_ context.Context,
	streamIDs []redis.XMessage,
) []interfaces.SubCtxMessage {
	var results []interfaces.SubCtxMessage
	for _, msg := range streamIDs {
		// Extract the "data" field from the message
		data, ok := msg.Values["data"]
		if !ok {
			continue
		}

		// Convert to bytes if it's a string
		var payload []byte
		switch v := data.(type) {
		case string:
			payload = []byte(v)
		case []byte:
			payload = v
		default:
			continue
		}

		ack := bredis.NewAck(
			s.client,
			s.cfg.GroupName,
			s.cfg.TopicName,
			msg.ID,
		)

		results = append(results, interfaces.SubCtxMessage{
			Payload: payload,
			Ack:     ack,
		})
	}
	return results
}

// autoclaim attempts to auto-claim pending messages from the stream.
//
// If auto_claim is disabled (set to 0), it returns an empty list and the same ID.
//
// # Arguments
//
//   - ctx: Context for cancellation
//   - autoClaimID: The ID to start auto-claiming from
//
// # Returns
//
// A tuple of (claimed message IDs, next auto-claim ID) or an error
func (s *Subscriber) autoclaim(
	ctx context.Context,
	autoClaimID string,
) ([]redis.XMessage, string, error) {
	if s.cfg.AutoClaim == 0 {
		return []redis.XMessage{}, autoClaimID, nil
	}

	messages, next, err := s.client.XAutoClaim(ctx, &redis.XAutoClaimArgs{
		Stream:   s.cfg.TopicName,
		Group:    s.cfg.GroupName,
		Consumer: s.cfg.ConsumerName,
		MinIdle:  time.Duration(s.cfg.AutoClaim) * time.Millisecond,
		Start:    autoClaimID,
		Count:    int64(s.cfg.NumFetch),
	}).Result()

	if err != nil {
		redisErr, ok := err.(redis.Error)
		if ok {
			brokerErr := errors.NewBrokerError(
				rediserrors.NewSubscribeAutoClaimError(&redisErr),
			)
			return nil, "", errors.SubBrokerError(brokerErr)
		}
		return nil, "", errors.SubBrokerError(errors.NewBrokerError(err))
	}

	return messages, next, nil
}

// Subscribe subscribes to a Redis stream and returns a channel of messages.
//
// Creates a consumer group if it doesn't exist, then continuously reads messages
// from the configured Redis stream. Each message is wrapped with an acknowledgment handler.
//
// The channel is closed when the subscription ends due to context cancellation or error.
//
// # Arguments
//
//   - ctx: Context for cancellation and timeouts
//
// # Returns
//
// A channel that yields SubCtxMessage containing payload and ack handler, or an error if subscription fails.
func (s *Subscriber) Subscribe(
	ctx context.Context,
) (<-chan interfaces.SubCtxMessage, error) {
	// Create the consumer group if it doesn't exist
	err := bredis.MakeStreamGroup(ctx, s.client, s.cfg.TopicName, s.cfg.GroupName)
	if err != nil {
		redisErr, ok := err.(redis.Error)
		if ok {
			brokerErr := errors.NewBrokerError(
				rediserrors.NewSubscribeGroupCreationError(&redisErr),
			)
			return nil, errors.SubBrokerError(brokerErr)
		}
		return nil, errors.SubBrokerError(errors.NewBrokerError(err))
	}

	ch := make(chan interfaces.SubCtxMessage)

	go func() {
		defer close(ch)

		autoClaimID := "0-0"

		for {
			select {
			case <-ctx.Done():
				return
			default:
			}

			// Run autoclaim and stream read in parallel
			autoClaimChan := make(chan struct {
				messages []redis.XMessage
				nextID   string
				err      error
			})

			streamReadChan := make(chan struct {
				messages []redis.XMessage
				err      error
			})

			go func() {
				messages, nextID, err := s.autoclaim(ctx, autoClaimID)
				autoClaimChan <- struct {
					messages []redis.XMessage
					nextID   string
					err      error
				}{messages, nextID, err}
			}()

			go func() {
				result, err := s.client.XReadGroup(ctx, &redis.XReadGroupArgs{
					Group:    s.cfg.GroupName,
					Consumer: s.cfg.ConsumerName,
					Streams:  []string{s.cfg.TopicName, ">"},
					Count:    int64(s.cfg.NumFetch),
					Block:    time.Duration(s.cfg.BlockTime) * time.Millisecond,
				}).Result()

				var messages []redis.XMessage
				if err == nil && len(result) > 0 {
					messages = result[0].Messages
				} else if err != redis.Nil && err != nil {
					// Only process actual errors, not nil results
					redisErr, ok := err.(redis.Error)
					if ok {
						streamReadChan <- struct {
							messages []redis.XMessage
							err      error
						}{nil, errors.SubBrokerError(
							errors.NewBrokerError(
								rediserrors.NewSubscribeReadError(&redisErr),
							),
						)}
						return
					}
					streamReadChan <- struct {
						messages []redis.XMessage
						err      error
					}{nil, errors.SubBrokerError(
						errors.NewBrokerError(err),
					)}
					return
				}

				streamReadChan <- struct {
					messages []redis.XMessage
					err      error
				}{messages, nil}
			}()

			// Wait for both operations to complete
			autoClaimResult := <-autoClaimChan
			streamReadResult := <-streamReadChan

			if autoClaimResult.err != nil {
				return
			}

			if streamReadResult.err != nil {
				return
			}

			// Update the autoclaim ID for the next iteration
			autoClaimID = autoClaimResult.nextID

			// Collect all message IDs (from autoclaim and stream read)
			var allMessages []redis.XMessage
			allMessages = append(allMessages, autoClaimResult.messages...)
			allMessages = append(allMessages, streamReadResult.messages...)

			// Process all messages
			values := s.handleStreamIds(ctx, allMessages)

			for _, value := range values {
				select {
				case ch <- value:
				case <-ctx.Done():
					return
				}
			}
		}
	}()

	return ch, nil
}

// Unsubscribe unsubscribes this consumer from the Redis stream consumer group.
//
// This implementation issues an XGROUP DELCONSUMER command to remove the
// configured consumer from the consumer group on the configured stream.
// While Redis streams do not require an explicit "unsubscribe" to stop
// receiving messages, this cleanup helps remove the consumer's pending
// entries from the group and free related server-side state.
//
// # Arguments
//
//   - ctx: Context for cancellation and timeouts
//
// # Returns
//
// An error if the unsubscription operation fails.
func (s *Subscriber) Unsubscribe(ctx context.Context) error {
	err := s.client.XGroupDelConsumer(
		ctx,
		s.cfg.TopicName,
		s.cfg.GroupName,
		s.cfg.ConsumerName,
	).Err()

	if err != nil {
		redisErr, ok := err.(redis.Error)
		if ok {
			return errors.UnSubBrokerError(
				errors.NewBrokerError(
					rediserrors.NewUnsubscribeError(&redisErr),
				),
			)
		}
		return errors.UnSubBrokerError(
			errors.NewBrokerError(err),
		)
	}

	return nil
}

// Verify that Subscriber implements the expected interfaces
var (
	_ interfaces.ISubCtxTrait = (*Subscriber)(nil)
)

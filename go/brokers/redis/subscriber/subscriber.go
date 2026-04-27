package subscriber

import (
	"context"
	"time"

	"github.com/redis/go-redis/v9"

	"github.com/hiroaki-yamamoto/object-transfer/go/errors"
	bredis "github.com/hiroaki-yamamoto/object-transfer/go/brokers/redis"
	"github.com/hiroaki-yamamoto/object-transfer/go/brokers/redis/config"
	rediserrors "github.com/hiroaki-yamamoto/object-transfer/go/brokers/redis/errors"

	"github.com/hiroaki-yamamoto/object-transfer/go/brokers/interfaces"
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

// handleStreamIDs processes stream IDs and extracts payloads with their acknowledgment handlers.
//
// Malformed entries (missing "data" field or unexpected type) are returned as
// SubBrokerMsg{Err: ..., Ack: ...} so callers can decide whether to acknowledge
// or drop them, preventing messages from accumulating in the pending-entry list.
//
// # Arguments
//
//   - _ (ctx): Context for cancellation
//   - streamIDs: The stream IDs to process
//
// # Returns
//
// A slice of SubBrokerMsg containing payloads or errors with their acknowledgment handlers.
func (s *Subscriber) handleStreamIDs(
	_ context.Context,
	streamIDs []redis.XMessage,
) []interfaces.SubBrokerMsg {
	var results []interfaces.SubBrokerMsg
	for _, msg := range streamIDs {
		// Always create the ack handler first so malformed entries can also be
		// returned with an Ack, letting callers decide whether to acknowledge or
		// drop them and avoid entries accumulating in the pending-entry list.
		ack := bredis.NewAck(
			s.client,
			s.cfg.GroupName,
			s.cfg.TopicName,
			msg.ID,
		)

		// Extract the "data" field from the message
		data, ok := msg.Values["data"]
		if !ok {
			results = append(results, interfaces.SubBrokerMsg{
				Err: errors.NewBrokerError(
					rediserrors.NewSubscribeMissingDataFieldError(msg.ID),
				),
				Ack: ack,
			})
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
			results = append(results, interfaces.SubBrokerMsg{
				Err: errors.NewBrokerError(
					rediserrors.NewSubscribeInvalidDataTypeError(msg.ID, data),
				),
				Ack: ack,
			})
			continue
		}

		results = append(results, interfaces.SubBrokerMsg{
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
) ([]redis.XMessage, string, *errors.BrokerError) {
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
		brokerErr := errors.NewBrokerError(
			rediserrors.NewSubscribeAutoClaimError(err),
		)
		return nil, "", brokerErr
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
// A channel that yields SubBrokerMsg containing payload and ack handler, or an error if subscription fails.
func (s *Subscriber) Subscribe(
	ctx context.Context,
) (<-chan interfaces.SubBrokerMsg, *errors.BrokerError) {
	// Create the consumer group if it doesn't exist
	err := bredis.MakeStreamGroup(ctx, s.client, s.cfg.TopicName, s.cfg.GroupName)
	if err != nil {
		return nil, errors.NewBrokerError(
			rediserrors.NewSubscribeGroupCreationError(err),
		)
	}

	ch := make(chan interfaces.SubBrokerMsg)

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
			type autoClaimResult struct {
				messages []redis.XMessage
				nextID   string
				err      *errors.BrokerError
			}
			type streamReadResult struct {
				messages []redis.XMessage
				err      *errors.BrokerError
			}

			autoClaimChan := make(chan autoClaimResult, 1)
			streamReadChan := make(chan streamReadResult, 1)

			go func() {
				messages, nextID, err := s.autoclaim(ctx, autoClaimID)
				autoClaimChan <- autoClaimResult{messages, nextID, err}
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
					streamReadChan <- streamReadResult{nil, errors.NewBrokerError(
						rediserrors.NewSubscribeReadError(err),
					)}
					return
				}

				streamReadChan <- streamReadResult{messages, nil}
			}()

			// Wait for both operations to complete; give up if context is cancelled.
			// Buffered channels ensure goroutines can always send and exit.
			var acResult autoClaimResult
			var srResult streamReadResult

			select {
			case acResult = <-autoClaimChan:
			case <-ctx.Done():
				return
			}

			select {
			case srResult = <-streamReadChan:
			case <-ctx.Done():
				return
			}

			if acResult.err != nil {
				select {
				case ch <- interfaces.SubBrokerMsg{Err: acResult.err}:
				case <-ctx.Done():
				}
				return
			}

			if srResult.err != nil {
				select {
				case ch <- interfaces.SubBrokerMsg{Err: srResult.err}:
				case <-ctx.Done():
				}
				return
			}

			// Update the autoclaim ID for the next iteration
			autoClaimID = acResult.nextID

			// Collect all message IDs (from autoclaim and stream read)
			var allMessages []redis.XMessage
			allMessages = append(allMessages, acResult.messages...)
			allMessages = append(allMessages, srResult.messages...)

			// Process all messages
			values := s.handleStreamIDs(ctx, allMessages)

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
// An UnSubError if the unsubscription operation fails.
func (s *Subscriber) Unsubscribe(ctx context.Context) *errors.UnSubError {
	err := s.client.XGroupDelConsumer(
		ctx,
		s.cfg.TopicName,
		s.cfg.GroupName,
		s.cfg.ConsumerName,
	).Err()

	if err != nil {
		return errors.UnSubBrokerError(
			errors.NewBrokerError(
				rediserrors.NewUnsubscribeError(err),
			),
		)
	}

	return nil
}

// Verify that Subscriber implements the expected interfaces
var (
	_ interfaces.ISubBroker = (*Subscriber)(nil)
)

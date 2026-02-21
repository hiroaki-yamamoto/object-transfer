package subscriber

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/vmihailenco/msgpack/v5"

	"github.com/hiroaki-yamamoto/object-transfer/go/format"
	"github.com/hiroaki-yamamoto/object-transfer/go/interfaces"
)

// Sub is a generic subscriber that deserializes messages and optionally acknowledges them.
//
// The subscriber relies on an ISubCtxTrait implementation for message retrieval and an
// ISubOpt provider for decoding and acknowledgment behavior.
//
// # Example
//
//	ctx := context.Background()
//	client, _ := nats.Connect("demo.nats.io")
//	js := jetstream.New(client)
//
//	type Event struct {
//	  ID   uint64 `json:"id" msgpack:"id"`
//	  Name string `json:"name" msgpack:"name"`
//	}
//
//	options := nats.NewAckSubOptions(format.FormatJSON, "events")
//	options.Subjects([]string{"events.user_created"})
//	options.DurableName("user-created")
//	options.AutoAck(false)
//
//	fetcher, _ := nats.NewSubFetcher(js, options)
//	subscriber := NewSub[Event](fetcher, fetcher, options)
//	messages, _ := subscriber.Subscribe(ctx)
//
//	for msg := range messages {
//	  if msg.Error != nil {
//	    log.Printf("error: %v\n", msg.Error)
//	    continue
//	  }
//	  if msg.Item != nil {
//	    fmt.Printf("received %+v\n", *msg.Item)
//	    msg.Ack.Ack(ctx)
//	  }
//	}
type Sub[T any] struct {
	ctx     interfaces.ISubCtxTrait
	unsub   interfaces.IUnSub
	options interfaces.ISubOpt
}

// NewSub creates a new subscriber using the provided context, unsubscribe handler, and
// subscription options.
//
// # Parameters
//   - ctx: Message retrieval context responsible for producing raw items.
//   - unsub: Unsubscribe handler to cancel the subscription when requested.
//   - options: Subscription behavior such as auto-ack and payload format.
func NewSub[T any](
	ctx interfaces.ISubCtxTrait,
	unsub interfaces.IUnSub,
	options interfaces.ISubOpt,
) *Sub[T] {
	return &Sub[T]{
		ctx:     ctx,
		unsub:   unsub,
		options: options,
	}
}

// Subscribe returns a channel of SubMessage containing decoded items and their acknowledgment handlers.
// When auto-acknowledgment is enabled, messages are acknowledged before being yielded to the consumer.
//
// # Parameters
//   - ctx: context for cancellation and timeouts
//
// # Returns
// A channel that yields SubMessage items containing decoded messages and their ack handlers,
// or an error if subscription fails.
func (s *Sub[T]) Subscribe(ctx context.Context) (<-chan interfaces.SubMessage[T], error) {
	// Get the raw messages from the context
	rawMessages, err := s.ctx.Subscribe(ctx)
	if err != nil {
		return nil, err
	}

	// Create output channel for decoded messages
	messages := make(chan interfaces.SubMessage[T])

	// Start goroutine to process and decode messages
	go func() {
		defer close(messages)

		for rawMsg := range rawMessages {
			// Auto-ack if enabled
			if s.options.GetAutoAck() {
				if err := rawMsg.Ack.Ack(ctx); err != nil {
					messages <- interfaces.SubMessage[T]{
						Item:  nil,
						Error: err,
						Ack:   rawMsg.Ack,
					}
					continue
				}
			}

			// Deserialize the message based on format
			var decodedItem T
			switch s.options.GetFormat() {
			case format.FormatMsgpack:
				if err := msgpack.Unmarshal(rawMsg.Payload, &decodedItem); err != nil {
					messages <- interfaces.SubMessage[T]{
						Item:  nil,
						Error: err,
						Ack:   rawMsg.Ack,
					}
					continue
				}
			case format.FormatJSON:
				if err := json.Unmarshal(rawMsg.Payload, &decodedItem); err != nil {
					messages <- interfaces.SubMessage[T]{
						Item:  nil,
						Error: err,
						Ack:   rawMsg.Ack,
					}
					continue
				}
			default:
				messages <- interfaces.SubMessage[T]{
					Item:  nil,
					Error: fmt.Errorf("unsupported format: %v", s.options.GetFormat()),
					Ack:   rawMsg.Ack,
				}
				continue
			}

			// Send decoded message
			messages <- interfaces.SubMessage[T]{
				Item:  &decodedItem,
				Error: nil,
				Ack:   rawMsg.Ack,
			}
		}
	}()

	return messages, nil
}

// Unsubscribe invokes the configured unsubscribe handler.
//
// # Parameters
//   - ctx: context for cancellation and timeouts
//
// # Returns
// An error if unsubscription fails.
func (s *Sub[T]) Unsubscribe(ctx context.Context) error {
	return s.unsub.Unsubscribe(ctx)
}

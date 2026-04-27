package subscriber

import (
	"context"
	"errors"

	goErrors "github.com/hiroaki-yamamoto/object-transfer/go/errors"
	"github.com/hiroaki-yamamoto/object-transfer/go/interfaces"
)

// Sub is a generic subscriber that deserializes messages and optionally acknowledges them.
//
// The subscriber relies on an ISubCtx implementation for message retrieval, an
// unmarshal function for decoding, and an Option for acknowledgment behavior.
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
//	natsOpts := nats.NewAckSubOptions("events")
//	natsOpts.Subjects("events.user_created")
//	natsOpts.DurableName("user-created")
//
//	fetcher, _ := nats.NewSubFetcher(js, natsOpts)
//	subOpts := NewOption().AutoAck(false)
//	subscriber, _ := NewSub[Event](fetcher, json.Unmarshal, fetcher, subOpts)
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
	ctx         interfaces.ISubCtx
	unmarshalFn func([]byte, any) error
	unsub       interfaces.IUnSub
	options     *Option
}

// NewSub creates a new subscriber using the provided context, unsubscribe handler, and
// subscription options.
//
// # Parameters
//   - ctx: Message retrieval context responsible for producing raw items.
//   - unsub: Unsubscribe handler to cancel the subscription when requested.
//   - options: Subscription behavior such as auto-ack and payload format.
func NewSub[T any](
	ctx interfaces.ISubCtx,
	unmarshalFn func([]byte, any) error,
	unsub interfaces.IUnSub,
	options *Option,
) (*Sub[T], *goErrors.SubError) {
	if ctx == nil {
		return nil, goErrors.NewSubError(errors.New("context is nil"))
	}
	if unmarshalFn == nil {
		return nil, goErrors.NewSubError(errors.New("unmarshalFn is nil"))
	}
	if unsub == nil {
		return nil, goErrors.NewSubError(errors.New("unsub is nil"))
	}
	if options == nil {
		options = NewOption()
	}
	return &Sub[T]{
		ctx:         ctx,
		unmarshalFn: unmarshalFn,
		unsub:       unsub,
		options:     options,
	}, nil
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
func (s *Sub[T]) Subscribe(ctx context.Context) (<-chan interfaces.SubMessage[T], *goErrors.SubError) {
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
			// Propagate transport-level errors from the raw subscription layer.
			if rawMsg.Err != nil {
				select {
				case messages <- interfaces.SubMessage[T]{
					Error: rawMsg.Err,
					Ack:   rawMsg.Ack,
				}:
				case <-ctx.Done():
					return
				}
				continue
			}

			// Deserialize the message based on unmarshal func (before auto-ack to avoid
			// acknowledging messages that cannot be decoded).
			var decodedItem T
			var decodeErr *goErrors.SubError
			if err := s.unmarshalFn(rawMsg.Payload, &decodedItem); err != nil {
				decodeErr = goErrors.SubDecodeError(err)
			}

			if decodeErr != nil {
				select {
				case messages <- interfaces.SubMessage[T]{
					Item:  nil,
					Error: decodeErr,
					Ack:   rawMsg.Ack,
				}:
				case <-ctx.Done():
					return
				}
				continue
			}

			// Auto-ack after successful decode
			if s.options.autoAck && rawMsg.Ack != nil {
				if err := rawMsg.Ack.Ack(ctx); err != nil {
					select {
					case messages <- interfaces.SubMessage[T]{
						Item:  nil,
						Error: goErrors.SubAckError(err),
						Ack:   rawMsg.Ack,
					}:
					case <-ctx.Done():
						return
					}
					continue
				}
			}

			// Allocate a new item per message to avoid the consumer observing
			// later iterations overwriting the value behind a shared pointer.
			item := new(T)
			*item = decodedItem

			// Send decoded message
			select {
			case messages <- interfaces.SubMessage[T]{
				Item:  item,
				Error: nil,
				Ack:   rawMsg.Ack,
			}:
			case <-ctx.Done():
				return
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
// An UnSubError if unsubscription fails.
func (s *Sub[T]) Unsubscribe(ctx context.Context) *goErrors.UnSubError {
	return s.unsub.Unsubscribe(ctx)
}

package subfetcher

import (
	"context"
	"errors"
	"fmt"

	"github.com/nats-io/nats.go"

	"github.com/hiroaki-yamamoto/object-transfer/go/interfaces"
	natstypes "github.com/hiroaki-yamamoto/object-transfer/go/nats"
)

// SubFetcher fetches pull-based JetStream messages using the configured stream options.
type SubFetcher struct {
	stream  nats.JetStreamContext
	options *AckSubOptions
}

// NewSubFetcher creates or reuses a JetStream stream based on the provided options.
//
// Parameters:
//   - ctx: Context for cancellation and timeouts
//   - jsCtx: JetStream context used to resolve or create the target stream
//   - opts: Configuration for the stream and durable pull consumer
//
// Returns:
// A new SubFetcher instance or an error if stream creation fails.
func NewSubFetcher(ctx context.Context, jsCtx nats.JetStreamContext, opts *AckSubOptions) (*SubFetcher, error) {
	_, err := jsCtx.AddStream(opts.streamConfig, nats.Context(ctx))
	if err != nil && err != nats.ErrStreamNameAlreadyInUse {
		return nil, err
	}
	return &SubFetcher{
		stream:  jsCtx,
		options: opts,
	}, nil
}

// Subscribe streams messages from the pull consumer, yielding their payloads along
// with the associated acknowledgment handles.
//
// This implements [interfaces.ISubCtxTrait].
func (f *SubFetcher) Subscribe(ctx context.Context) (<-chan interfaces.SubCtxMessage, error) {
	if len(f.options.streamConfig.Subjects) == 0 {
		err := NewSubFetcherError(
			fmt.Errorf("stream must have at least one subject"),
		)
		return nil, err
	}
	subject := f.options.streamConfig.Subjects[0]

	sub, err := f.stream.PullSubscribe(
		subject,
		f.options.consumerConfig.Durable,
		nats.Context(ctx),
	)
	if err != nil {
		return nil, err
	}

	ch := make(chan interfaces.SubCtxMessage)
	go func() {
		defer close(ch)
		for {
			select {
			case <-ctx.Done():
				return
			default:
				msgs, err := sub.Fetch(1, nats.Context(ctx))
				if err != nil {
					if errors.Is(err, nats.ErrTimeout) {
						continue
					}
					// Emit the error downstream before exiting
					select {
					case ch <- interfaces.SubCtxMessage{Err: NewSubFetcherError(err)}:
					case <-ctx.Done():
					}
					return
				}
				for _, msg := range msgs {
					select {
					case ch <- interfaces.SubCtxMessage{
						Payload: msg.Data,
						Ack:     natstypes.NewAck(msg),
					}:
					case <-ctx.Done():
						return
					}
				}
			}
		}
	}()

	return ch, nil
}

// Unsubscribe deletes the durable consumer associated with this fetcher.
//
// This implements [interfaces.IUnSub].
func (f *SubFetcher) Unsubscribe(ctx context.Context) error {
	if f.options.consumerConfig.Durable == "" {
		return nil
	}
	return f.stream.DeleteConsumer(
		f.options.streamConfig.Name,
		f.options.consumerConfig.Durable,
		nats.Context(ctx),
	)
}

var _ interfaces.ISubCtxTrait = (*SubFetcher)(nil)
var _ interfaces.IUnSub = (*SubFetcher)(nil)

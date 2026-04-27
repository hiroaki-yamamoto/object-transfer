package subfetcher

import (
	"context"
	stderrors "errors"
	"fmt"

	"github.com/nats-io/nats.go"

	otErrors "github.com/hiroaki-yamamoto/object-transfer/go/errors"
	"github.com/hiroaki-yamamoto/object-transfer/go/interfaces"
	natstypes "github.com/hiroaki-yamamoto/object-transfer/go/nats"
	"github.com/hiroaki-yamamoto/object-transfer/go/unsub"
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
// This implements [interfaces.ISubCtx].
func (f *SubFetcher) Subscribe(ctx context.Context) (<-chan interfaces.SubCtxMessage, *otErrors.SubError) {
	if len(f.options.streamConfig.Subjects) == 0 {
		err := NewSubFetcherError(
			fmt.Errorf("stream must have at least one subject"),
		)
		return nil, otErrors.SubBrokerError(
			otErrors.NewBrokerError(err),
		)
	}
	if len(f.options.streamConfig.Subjects) > 1 {
		err := NewSubFetcherError(
			fmt.Errorf("multiple subjects configured on stream; only a single subject is supported"),
		)
		return nil, otErrors.SubBrokerError(
			otErrors.NewBrokerError(err),
		)
	}
	subject := f.options.streamConfig.Subjects[0]

	sub, err := f.stream.PullSubscribe(
		subject,
		f.options.consumerConfig.Durable,
		nats.Context(ctx),
	)
	if err != nil {
		return nil, otErrors.SubBrokerError(
			otErrors.NewBrokerError(NewSubFetcherError(err)),
		)
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
					if stderrors.Is(err, nats.ErrTimeout) {
						continue
					}
					// Emit the error downstream before exiting
					select {
					case ch <- interfaces.SubCtxMessage{Err: otErrors.SubBrokerError(
						otErrors.NewBrokerError(NewSubFetcherError(err)),
					)}:
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
func (f *SubFetcher) Unsubscribe(ctx context.Context) *otErrors.UnSubError {
	if f.options.consumerConfig.Durable == "" {
		return nil
	}
	err := f.stream.DeleteConsumer(
		f.options.streamConfig.Name,
		f.options.consumerConfig.Durable,
		nats.Context(ctx),
	)
	if err != nil {
		return otErrors.UnSubBrokerError(otErrors.NewBrokerError(err))
	}
	return nil
}

var _ interfaces.ISubCtx = (*SubFetcher)(nil)
var _ unsub.IUnSub = (*SubFetcher)(nil)

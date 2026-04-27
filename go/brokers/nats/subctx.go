package nats

import (
	"context"
	stderrors "errors"

	natssdk "github.com/nats-io/nats.go"

	otErrors "github.com/hiroaki-yamamoto/object-transfer/go/errors"
	"github.com/hiroaki-yamamoto/object-transfer/go/brokers/interfaces"
)

// PushSubCtx wraps a push-based NATS [nats.Subscription] and implements
// [interfaces.ISubBroker].
type PushSubCtx struct {
	sub *natssdk.Subscription
}

// Subscribe returns a channel that yields messages from the push subscription.
// The channel is closed when ctx is cancelled or the subscription encounters
// an error.
func (s *PushSubCtx) Subscribe(
	ctx context.Context,
) (<-chan interfaces.SubBrokerMsg, *otErrors.SubError) {
	ch := make(chan interfaces.SubBrokerMsg)
	go func() {
		defer close(ch)
		for {
			msg, err := s.sub.NextMsgWithContext(ctx)
			if err != nil {
				// Treat context cancellation and deadline exceeded as normal termination.
				if stderrors.Is(err, context.Canceled) ||
					stderrors.Is(err, context.DeadlineExceeded) ||
					stderrors.Is(ctx.Err(), context.Canceled) ||
					stderrors.Is(ctx.Err(), context.DeadlineExceeded) {
					return
				}
				// Propagate transport or other errors to the subscriber before exiting.
				select {
				case ch <- interfaces.SubBrokerMsg{Err: otErrors.SubBrokerError(
					otErrors.NewBrokerError(err),
				)}:
				case <-ctx.Done():
				}
				return
			}
			select {
			case ch <- interfaces.SubBrokerMsg{
				Payload: msg.Data,
				Ack:     NewAck(msg),
			}:
			case <-ctx.Done():
				return
			}
		}
	}()
	return ch, nil
}

// NewPushSubCtx creates a new [PushSubCtx] wrapping the given
// [nats.Subscription].
func NewPushSubCtx(sub *natssdk.Subscription) *PushSubCtx {
	return &PushSubCtx{sub: sub}
}

// PullSubCtx wraps a pull-based NATS [nats.Subscription] and implements
// [interfaces.ISubBroker].
type PullSubCtx struct {
	sub *natssdk.Subscription
}

// Subscribe returns a channel that yields messages fetched from the pull
// subscription one at a time. The channel is closed when ctx is cancelled or
// the subscription encounters an error.
func (s *PullSubCtx) Subscribe(
	ctx context.Context,
) (<-chan interfaces.SubBrokerMsg, *otErrors.SubError) {
	ch := make(chan interfaces.SubBrokerMsg)
	go func() {
		defer close(ch)
		for {
			select {
			case <-ctx.Done():
				return
			default:
				msgs, err := s.sub.Fetch(1, natssdk.Context(ctx))
				if err != nil {
					if stderrors.Is(err, natssdk.ErrTimeout) {
						continue
					}
					// Surface non-timeout fetch errors to the consumer so they can
					// distinguish broker/subscription failures from clean cancellation.
					select {
					case ch <- interfaces.SubBrokerMsg{Err: otErrors.SubBrokerError(
						otErrors.NewBrokerError(err),
					)}:
					case <-ctx.Done():
					}
					return
				}
				for _, msg := range msgs {
					select {
					case ch <- interfaces.SubBrokerMsg{
						Payload: msg.Data,
						Ack:     NewAck(msg),
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

// NewPullSubCtx creates a new [PullSubCtx] wrapping the given
// [nats.Subscription].
func NewPullSubCtx(sub *natssdk.Subscription) *PullSubCtx {
	return &PullSubCtx{sub: sub}
}

var _ interfaces.ISubBroker = (*PushSubCtx)(nil)
var _ interfaces.ISubBroker = (*PullSubCtx)(nil)

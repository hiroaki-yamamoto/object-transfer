package nats

import (
	"context"

	"github.com/nats-io/nats.go"

	"github.com/hiroaki-yamamoto/object-transfer/go/errors"
)

// Ack wraps a [nats.Msg] and implements [interfaces.IAck].
type Ack struct {
	msg *nats.Msg
}

// Ack acknowledges the NATS message using the provided context for
// cancellation and timeout control.
func (a *Ack) Ack(ctx context.Context) *errors.AckError {
	err := a.msg.AckSync(nats.Context(ctx))
	if err != nil {
		return errors.AckBrokerError(errors.NewBrokerError(err))
	}
	return nil
}

// NewAck creates a new [Ack] wrapping the given [nats.Msg].
func NewAck(msg *nats.Msg) *Ack {
	return &Ack{msg: msg}
}

package nats

import (
	"context"

	"github.com/nats-io/nats.go"
)

// Ack wraps a [nats.Msg] and implements [interfaces.IAck].
type Ack struct {
	msg *nats.Msg
}

// Ack acknowledges the NATS message using the provided context for
// cancellation and timeout control.
func (a *Ack) Ack(ctx context.Context) error {
	return a.msg.AckSync(nats.Context(ctx))
}

// NewAck creates a new [Ack] wrapping the given [nats.Msg].
func NewAck(msg *nats.Msg) *Ack {
	return &Ack{msg: msg}
}

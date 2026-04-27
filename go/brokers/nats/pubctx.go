package nats

import (
	"context"

	natssdk "github.com/nats-io/nats.go"

	otErrors "github.com/hiroaki-yamamoto/object-transfer/go/errors"
	"github.com/hiroaki-yamamoto/object-transfer/go/brokers/interfaces"
)

// PubCtx wraps a NATS JetStream context and implements [interfaces.IPubBroker].
type PubCtx struct {
	js natssdk.JetStreamContext
}

// Publish sends raw bytes to the given topic via the JetStream context.
func (p *PubCtx) Publish(ctx context.Context, topic string, payload []byte) *otErrors.PubError {
	_, err := p.js.Publish(topic, payload, natssdk.Context(ctx))
	if err != nil {
		return otErrors.PubBrokerError(otErrors.NewBrokerError(err))
	}
	return nil
}

// NewPubCtx creates a new [PubCtx] wrapping the given [nats.JetStreamContext].
func NewPubCtx(js natssdk.JetStreamContext) *PubCtx {
	return &PubCtx{js: js}
}

var _ interfaces.IPubBroker = (*PubCtx)(nil)

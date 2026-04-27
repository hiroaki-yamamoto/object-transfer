package interfaces

import (
	"context"

	"github.com/hiroaki-yamamoto/object-transfer/go/errors"
)

// IPubBroker is a context capable of publishing raw byte payloads.
type IPubBroker interface {
	// Publish sends a raw payload to a subject on the underlying broker.
	//
	// Parameters:
	// - ctx: context for cancellation and timeouts
	// - topic: Subject or channel name the payload should be delivered to
	// - payload: Serialized bytes to forward to the transport
	//
	// Returns an error if publishing fails.
	Publish(ctx context.Context, topic string, payload []byte) *errors.BrokerError
}

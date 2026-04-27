package interfaces

import (
	"context"

	"github.com/hiroaki-yamamoto/object-transfer/go/ack"
	"github.com/hiroaki-yamamoto/object-transfer/go/errors"
)

// SubBrokerMsg represents a raw message received from a subscription with its acknowledgment handler.
// If Err is non-nil, the message represents a retrieval error from the transport layer
// and Payload will be nil.
type SubBrokerMsg struct {
	Payload []byte
	Ack     ack.IAck
	Err     *errors.BrokerError
}

// ISubBroker is a context capable of producing a stream of raw messages with ack handles.
type ISubBroker interface {
	// Subscribe returns a channel of SubBrokerMsg containing raw byte payloads and their acknowledgment handlers.
	// The channel is closed when the subscription ends or the provided context is done.
	//
	// Parameters:
	// - ctx: context for cancellation and timeouts
	//
	// Returns a channel that yields messages, or an error if the subscription cannot be established.
	Subscribe(ctx context.Context) (<-chan SubBrokerMsg, *errors.BrokerError)
}

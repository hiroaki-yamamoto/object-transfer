package interfaces

import "context"

// SubCtxMessage represents a raw message received from a subscription with its acknowledgment handler.
type SubCtxMessage struct {
	Payload []byte
	Ack     IAck
}

// ISubCtxTrait is a context capable of producing a stream of raw messages with ack handles.
type ISubCtxTrait interface {
	// Subscribe returns a channel of SubCtxMessage containing raw byte payloads and their acknowledgment handlers.
	// The channel is closed when the subscription ends or the provided context is done.
	//
	// Parameters:
	// - ctx: context for cancellation and timeouts
	//
	// Returns a channel that yields messages, or an error if the subscription cannot be established.
	Subscribe(ctx context.Context) (<-chan SubCtxMessage, error)
}

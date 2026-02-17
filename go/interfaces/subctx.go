package interfaces

import "context"

// SubCtxMessage represents a raw message received from a subscription with its acknowledgment handler.
type SubCtxMessage struct {
	Payload []byte
	Ack     AckTrait
}

// SubCtxTrait is a context capable of producing a stream of raw messages with ack handles.
type SubCtxTrait interface {
	// Subscribe returns a channel of SubCtxMessage containing raw byte payloads and their acknowledgment handlers.
	// The channel is closed when the subscription ends.
	// Errors during message retrieval are sent as part of the channel.
	//
	// Parameters:
	// - ctx: context for cancellation and timeouts
	//
	// Returns a channel that yields messages or an error if subscription fails.
	Subscribe(ctx context.Context) (<-chan SubCtxMessage, error)
}

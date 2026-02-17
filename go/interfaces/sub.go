package interfaces

import "context"

// SubMessage represents a message received from a subscription along with its acknowledgment handler.
type SubMessage struct {
	Item any
	Ack  IAck
}

// ISubTrait is a subscription interface returning a stream of decoded items and ack handles.
type ISubTrait interface {
	// Subscribe returns a channel of SubMessage containing decoded items and their acknowledgment handlers.
	// The channel is closed when the subscription ends.
	// Errors during deserialization or message retrieval are sent as part of the channel.
	//
	// Parameters:
	// - ctx: context for cancellation and timeouts
	//
	// Returns a channel that yields messages or an error if subscription fails.
	Subscribe(ctx context.Context) (<-chan SubMessage, error)
}

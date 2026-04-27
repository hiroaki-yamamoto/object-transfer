package subscriber

import (
	"context"

	"github.com/hiroaki-yamamoto/object-transfer/go/ack"
	"github.com/hiroaki-yamamoto/object-transfer/go/errors"
)

// SubMessage represents a message received from a subscription along with its acknowledgment handler.
type SubMessage[T any] struct {
	Item  *T
	Error *errors.SubError
	Ack   ack.IAck
}

// ISub is a subscription interface returning a stream of decoded items and ack handles.
type ISub[T any] interface {
	// Subscribe returns a channel of SubMessage containing decoded items and their acknowledgment handlers.
	// The channel is closed when the subscription ends.
	// Errors during deserialization or message retrieval are sent as part of the channel.
	//
	// Parameters:
	// - ctx: context for cancellation and timeouts
	//
	// Returns a channel that yields messages or an error if subscription fails.
	Subscribe(ctx context.Context) (<-chan SubMessage[T], *errors.SubError)
}

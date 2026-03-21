package interfaces

import (
	"context"

	"github.com/hiroaki-yamamoto/object-transfer/go/errors"
)

// IAck provides the ability to acknowledge receipt of a message.
type IAck interface {
	// Ack acknowledges the receipt of a message.
	//
	// Parameters:
	// - ctx: context for cancellation and timeouts
	//
	// Returns an error if acknowledgment fails.
	Ack(ctx context.Context) *errors.AckError
}

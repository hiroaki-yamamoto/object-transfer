package interfaces

import "context"

// IAck provides the ability to acknowledge receipt of a message.
type IAck interface {
	// Ack acknowledges the receipt of a message.
	//
	// Parameters:
	// - ctx: context for cancellation and timeouts
	//
	// Returns an error if acknowledgment fails.
	Ack(ctx context.Context) error
}

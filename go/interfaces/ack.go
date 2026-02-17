package interfaces

import "context"

// AckTrait provides the ability to acknowledge receipt of a message.
type AckTrait interface {
	// Ack acknowledges the receipt of a message.
	//
	// Parameters:
	// - ctx: context for cancellation and timeouts
	//
	// Returns an error if acknowledgment fails.
	Ack(ctx context.Context) error
}

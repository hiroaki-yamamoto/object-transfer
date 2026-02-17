package interfaces

import "context"

// UnSubTrait allows canceling a subscription.
type UnSubTrait interface {
	// Unsubscribe cancels the subscription.
	//
	// Parameters:
	// - ctx: context for cancellation and timeouts
	//
	// Returns an error if unsubscription fails.
	Unsubscribe(ctx context.Context) error
}

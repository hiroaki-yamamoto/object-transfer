package interfaces

import "context"

// IUnSub allows canceling a subscription.
type IUnSub interface {
	// Unsubscribe cancels the subscription.
	//
	// Parameters:
	// - ctx: context for cancellation and timeouts
	//
	// Returns an error if unsubscription fails.
	Unsubscribe(ctx context.Context) error
}

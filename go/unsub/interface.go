package unsub

import (
	"context"

	"github.com/hiroaki-yamamoto/object-transfer/go/errors"
)

// IUnSub allows canceling a subscription.
type IUnSub interface {
	// Unsubscribe cancels the subscription.
	//
	// Parameters:
	// - ctx: context for cancellation and timeouts
	//
	// Returns an error if unsubscription fails.
	Unsubscribe(ctx context.Context) *errors.UnSubError
}

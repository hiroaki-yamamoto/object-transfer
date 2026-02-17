package interfaces

import "context"

// IPublish is an abstraction for publishing typed items.
//
// Implementors handle serialization and delivery to a concrete backend.
type IPublish interface {
	// IPublish sends a serializable item through the implementor.
	//
	// Parameters:
	// - ctx: context for cancellation and timeouts
	// - obj: The typed item to serialize and send to the backing transport
	//
	// Returns an error if publishing fails.
	Publish(ctx context.Context, obj interface{}) error
}

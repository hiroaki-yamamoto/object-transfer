package publisher

import (
	"context"

	"github.com/hiroaki-yamamoto/object-transfer/go/errors"
	"github.com/hiroaki-yamamoto/object-transfer/go/interfaces"
)

// Pub is a generic publisher for serializable messages using a pluggable context.
//
// The publisher encodes messages according to the configured marshal function and
// delegates the actual publish call to an injected IPubBroker so it can
// work with different backends.
//
// # Example
//
//	ctx := context.Background()
//	natsConn, _ := nats.Connect("demo.nats.io")
//	js, _ := natsConn.JetStream()
//
//	type UserCreated struct {
//	  ID   uint64 `json:"id" msgpack:"id"`
//	  Name string `json:"name" msgpack:"name"`
//	}
//
//	publisher := publisher.NewPub[UserCreated](js, "events.user_created", json.Marshal)
//
//	event := UserCreated{ID: 42, Name: "Jane Doe"}
//	publisher.Publish(ctx, &event)
type Pub[T any] struct {
	ctx     interfaces.IPubBroker
	subject string
	marshal func(v any) ([]byte, error)
}

// NewPub creates a new publisher for the given subject and serialization format.
//
// # Parameters
//   - ctx: Backend publish context that delivers serialized bytes.
//   - subject: Destination subject or topic to send messages to.
//   - marshal: Function used to serialize published items into bytes.
func NewPub[T any](
	ctx interfaces.IPubBroker,
	subject string,
	marshal func(v any) ([]byte, error),
) *Pub[T] {
	return &Pub[T]{
		ctx:     ctx,
		subject: subject,
		marshal: marshal,
	}
}

// Publish serializes the provided object and publishes it to the configured
// subject using the underlying context.
//
// # Parameters
//   - ctx: context for cancellation and timeouts
//   - obj: The typed value to encode and send to the subject
func (p *Pub[T]) Publish(ctx context.Context, obj interface{}) *errors.PubError {
	payload, err := p.marshal(obj)
	if err != nil {
		return errors.PubEncodeError(err)
	}

	pubErr := p.ctx.Publish(ctx, p.subject, payload)
	if pubErr != nil {
		return pubErr
	}

	return nil
}

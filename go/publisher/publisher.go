package publisher

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/vmihailenco/msgpack/v5"

	"github.com/hiroaki-yamamoto/object-transfer/go/errors"
	"github.com/hiroaki-yamamoto/object-transfer/go/format"
	"github.com/hiroaki-yamamoto/object-transfer/go/interfaces"
)

// Pub is a generic publisher for serializable messages using a pluggable context.
//
// The publisher encodes messages according to the configured Format and
// delegates the actual publish call to an injected IPubCtx so it can
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
//	publisher := publisher.NewPub[UserCreated](js, "events.user_created", format.FormatJSON)
//
//	event := UserCreated{ID: 42, Name: "Jane Doe"}
//	publisher.Publish(ctx, &event)
type Pub[T any] struct {
	ctx     interfaces.IPubCtx
	subject string
	format  format.Format
}

// NewPub creates a new publisher for the given subject and serialization format.
//
// # Parameters
//   - ctx: Backend publish context that delivers serialized bytes.
//   - subject: Destination subject or topic to send messages to.
//   - format: Serialization format used when encoding published items.
func NewPub[T any](
	ctx interfaces.IPubCtx,
	subject string,
	fmtType format.Format,
) *Pub[T] {
	return &Pub[T]{
		ctx:     ctx,
		subject: subject,
		format:  fmtType,
	}
}

// Publish serializes the provided object and publishes it to the configured
// subject using the underlying context.
//
// # Parameters
//   - ctx: context for cancellation and timeouts
//   - obj: The typed value to encode and send to the subject
func (p *Pub[T]) Publish(ctx context.Context, obj *T) error {
	var payload []byte
	var err error

	switch p.format {
	case format.FormatMsgpack:
		payload, err = msgpack.Marshal(obj)
		if err != nil {
			return errors.PubMessagePackEncodeError(err)
		}
	case format.FormatJSON:
		payload, err = json.Marshal(obj)
		if err != nil {
			return errors.PubJsonError(err)
		}
	default:
		return errors.NewPubError(fmt.Errorf("unsupported format: %v", p.format))
	}

	err = p.ctx.Publish(ctx, p.subject, payload)
	if err != nil {
		return errors.PubBrokerError(errors.NewBrokerError(err))
	}

	return nil
}

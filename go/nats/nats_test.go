package nats_test

import (
	"context"
	"fmt"
	"time"

	natssdk "github.com/nats-io/nats.go"
	. "github.com/onsi/ginkgo/v2"
	. "github.com/onsi/gomega"

	"github.com/hiroaki-yamamoto/object-transfer/go/format"
	"github.com/hiroaki-yamamoto/object-transfer/go/nats"
	"github.com/hiroaki-yamamoto/object-transfer/go/nats/subfetcher"
	"github.com/hiroaki-yamamoto/object-transfer/go/publisher"
	"github.com/hiroaki-yamamoto/object-transfer/go/subscriber"
)

// MyObj is a simple test struct for serialization testing
type MyObj struct {
	Field string `json:"field" msgpack:"field"`
}

// setup initializes a NATS connection, JetStream context, publisher, and subscriber
func setup(
	ctx context.Context,
	fmtType format.Format,
) (*publisher.Pub[MyObj], *subscriber.Sub[MyObj], error) {
	// Connect to NATS
	client, err := natssdk.Connect(
		"127.0.0.1:4222",
		natssdk.RetryOnFailedConnect(true),
		natssdk.MaxReconnects(5),
	)
	if err != nil {
		return nil, nil, err
	}

	// Create JetStream context
	js, err := client.JetStream()
	if err != nil {
		return nil, nil, err
	}

	// Create a unique name based on the format
	name := fmt.Sprintf("object_transfer_%s", fmtType)

	// Create publisher
	pubCtx := nats.NewPubCtx(js)
	pub := publisher.NewPub[MyObj](pubCtx, name, fmtType)

	// Create subscriber options
	opts := subfetcher.NewAckSubOptions(fmtType, name).
		Subjects(name).
		DurableName(name)

	// Create SubFetcher
	fetcher, err := subfetcher.NewSubFetcher(ctx, js, opts)
	if err != nil {
		return nil, nil, err
	}

	// Create subscriber
	sub := subscriber.NewSub[MyObj](fetcher, fetcher, opts)

	return pub, sub, nil
}

var _ = Describe("Nats", func() {
	testRoundtrip := func(fmtType format.Format) func() {
		return func() {
			ctx := context.Background()
			ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
			defer cancel()
			pub, sub, err := setup(ctx, fmtType)
			Expect(err).NotTo(HaveOccurred())

			obj := MyObj{Field: "value"}

			// Channel to receive the result from the subscription goroutine
			resultCh := make(chan MyObj)
			errCh := make(chan error)

			// Spawn a goroutine to subscribe and receive messages
			go func() {
				messages, err := sub.Subscribe(ctx)
				if err != nil {
					errCh <- err
					return
				}

				// Read the first message
				for msg := range messages {
					if msg.Error != nil {
						errCh <- msg.Error
						return
					}
					if msg.Item != nil {
						// Acknowledge the message
						if msg.Ack != nil {
							msg.Ack.Ack(ctx)
						}
						resultCh <- *msg.Item
						return
					}
				}
			}()

			// Publish the object
			err = pub.Publish(ctx, &obj)
			Expect(err).NotTo(HaveOccurred())

			// Wait for the received object
			select {
			case recv := <-resultCh:
				Expect(recv).To(Equal(obj))
			case err := <-errCh:
				Fail(fmt.Sprintf("subscription error: %v", err))
			case <-ctx.Done():
				Fail(fmt.Sprintf("context canceled while waiting for message: %v", ctx.Err()))
			}

			// Unsubscribe
			err = sub.Unsubscribe(ctx)
			Expect(err).NotTo(HaveOccurred())
		}
	}

	Describe("MessagePack format", func() {
		It("should publish and subscribe to a message", testRoundtrip(format.FormatMsgpack))
	})

	Describe("JSON format", func() {
		It("should publish and subscribe to a message", testRoundtrip(format.FormatJSON))
	})
})

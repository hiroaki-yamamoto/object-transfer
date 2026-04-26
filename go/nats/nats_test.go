package nats_test

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	natssdk "github.com/nats-io/nats.go"
	. "github.com/onsi/ginkgo/v2"
	. "github.com/onsi/gomega"
	"github.com/vmihailenco/msgpack/v5"

	"github.com/hiroaki-yamamoto/object-transfer/go/nats"
	"github.com/hiroaki-yamamoto/object-transfer/go/nats/subfetcher"
	"github.com/hiroaki-yamamoto/object-transfer/go/publisher"
	"github.com/hiroaki-yamamoto/object-transfer/go/subscriber"
)

// MyObj is a simple test struct for serialization testing
type MyObj struct {
	Field string `json:"field" msgpack:"field"`
}

func uniqueStreamName(name string) string {
	now := time.Now().UnixNano()
	return fmt.Sprintf("object_transfer_nats_%s_%d", name, now)
}

// setup initializes a NATS connection, JetStream context, publisher, and subscriber
func setup(
	ctx context.Context,
	name string,
	marshal func(any) ([]byte, error),
	unmarshal func([]byte, any) error,
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

	uniqueName := uniqueStreamName(name)

	// Create publisher
	pubCtx := nats.NewPubCtx(js)
	pub := publisher.NewPub[MyObj](pubCtx, uniqueName, marshal)

	// Create subscriber options
	opts := subfetcher.NewAckSubOptions(uniqueName).
		Subjects(uniqueName).
		DurableName(uniqueName)

	// Create SubFetcher
	fetcher, err := subfetcher.NewSubFetcher(ctx, js, opts)
	if err != nil {
		return nil, nil, err
	}

	// Create subscriber
	subOpts := subscriber.NewOption().AutoAck(false)
	sub, subErr := subscriber.NewSub[MyObj](fetcher, unmarshal, fetcher, subOpts)
	if subErr != nil {
		return nil, nil, subErr
	}

	return pub, sub, nil
}

var _ = Describe("Nats", func() {
	testRoundtrip := func(name string, marshal func(any) ([]byte, error), unmarshal func([]byte, any) error) func() {
		return func() {
			ctx := context.Background()
			ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
			defer cancel()
			pub, sub, err := setup(ctx, name, marshal, unmarshal)
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
		It("should publish and subscribe to a message", testRoundtrip("object_transfer_msgpack", msgpack.Marshal, msgpack.Unmarshal))
	})

	Describe("JSON format", func() {
		It("should publish and subscribe to a message", testRoundtrip("object_transfer_json", json.Marshal, json.Unmarshal))
	})
})

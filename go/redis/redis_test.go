package redis_test

import (
	"context"
	"fmt"
	"time"

	. "github.com/onsi/ginkgo/v2"
	. "github.com/onsi/gomega"
	"github.com/redis/go-redis/v9"

	"github.com/hiroaki-yamamoto/object-transfer/go/format"
	pubpkg "github.com/hiroaki-yamamoto/object-transfer/go/publisher"
	redisconfig "github.com/hiroaki-yamamoto/object-transfer/go/redis/config"
	"github.com/hiroaki-yamamoto/object-transfer/go/redis/publisher"
	"github.com/hiroaki-yamamoto/object-transfer/go/redis/subscriber"
	subpkg "github.com/hiroaki-yamamoto/object-transfer/go/subscriber"
)

// MyObj is a test struct for serialization testing
type MyObj struct {
	Field string `json:"field" msgpack:"field"`
}

// TestSubOptions implements the ISubOpt interface for testing
type TestSubOptions struct {
	format  format.Format
	autoAck bool
}

func (tso *TestSubOptions) GetAutoAck() bool {
	return tso.autoAck
}

func (tso *TestSubOptions) GetFormat() format.Format {
	return tso.format
}

// uniqueStreamName generates a unique stream name based on the format and current timestamp
func uniqueStreamName(fmtType format.Format) string {
	now := time.Now().UnixMilli()
	return fmt.Sprintf("object_transfer_redis_%s_%d", fmtType, now)
}

// setup creates a publisher and subscriber for testing
func setup(ctx context.Context, fmtType format.Format) (*pubpkg.Pub[MyObj], *subpkg.Sub[MyObj], error) {
	// Connect to Redis
	client := redis.NewClient(&redis.Options{
		Addr: "127.0.0.1:6379",
	})

	// Test connection
	if err := client.Ping(ctx).Err(); err != nil {
		return nil, nil, fmt.Errorf("failed to connect to Redis: %w", err)
	}

	streamName := uniqueStreamName(fmtType)
	publisherGroup := fmt.Sprintf("%s_publisher", streamName)
	subscriberGroup := fmt.Sprintf("%s_subscriber", streamName)
	subscriberConsumer := fmt.Sprintf("%s_consumer", streamName)

	// Create publisher
	publisherCfg := redisconfig.NewPublisherConfig().
		WithGroupName(publisherGroup).
		WithStreamLength(1000)

	publ := publisher.New(client, publisherCfg)

	// Create subscriber
	subscriberCfg := redisconfig.NewSubscriberConfig(streamName).
		WithGroupName(subscriberGroup).
		WithConsumerName(subscriberConsumer).
		WithNumFetch(1).
		WithBlockTime(500)

	subsc := subscriber.New(client, subscriberCfg)

	// Create typed publisher and subscriber
	options := &TestSubOptions{
		format:  fmtType,
		autoAck: true,
	}

	pub := pubpkg.NewPub[MyObj](publ, streamName, fmtType)
	sub := subpkg.NewSub[MyObj](subsc, subsc, options)

	return pub, sub, nil
}

// roundtrip tests the publish/subscribe roundtrip
func roundtrip(ctx context.Context, fmtType format.Format) {
	pub, sub, err := setup(ctx, fmtType)
	Expect(err).NotTo(HaveOccurred())
	Expect(pub).NotTo(BeNil())
	Expect(sub).NotTo(BeNil())

	// Create the object to publish
	obj := MyObj{Field: "value"}

	// Subscribe in a goroutine
	receivedChan := make(chan *MyObj, 1)
	errChan := make(chan error, 1)

	go func() {
		messages, err := sub.Subscribe(ctx)
		if err != nil {
			errChan <- err
			return
		}

		for msg := range messages {
			if msg.Error != nil {
				errChan <- msg.Error
				return
			}
			if msg.Item != nil {
				receivedChan <- msg.Item
				return
			}
		}
	}()

	// Give subscriber time to subscribe
	time.Sleep(100 * time.Millisecond)

	// Publish the object
	err = pub.Publish(ctx, &obj)
	Expect(err).NotTo(HaveOccurred())

	// Wait for the message to be received or timeout
	select {
	case received := <-receivedChan:
		Expect(received).NotTo(BeNil())
		Expect(received.Field).To(Equal(obj.Field))
	case err := <-errChan:
		Fail(fmt.Sprintf("Received error: %v", err))
	case <-time.After(5 * time.Second):
		Fail("Timeout waiting for message")
	}

	// Unsubscribe
	err = sub.Unsubscribe(ctx)
	Expect(err).NotTo(HaveOccurred())
}

var _ = Describe("Redis", func() {
	var ctx context.Context

	BeforeEach(func() {
		ctx = context.Background()
	})

	It("should roundtrip with MessagePack format", func() {
		roundtrip(ctx, format.FormatMsgpack)
	})

	It("should roundtrip with JSON format", func() {
		roundtrip(ctx, format.FormatJSON)
	})
})

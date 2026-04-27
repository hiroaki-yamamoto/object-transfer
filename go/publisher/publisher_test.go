package publisher_test

import (
	"context"
	"encoding/json"
	"fmt"

	. "github.com/onsi/ginkgo/v2"
	. "github.com/onsi/gomega"
	"github.com/vmihailenco/msgpack/v5"

	"github.com/hiroaki-yamamoto/object-transfer/go/errors"
	"github.com/hiroaki-yamamoto/object-transfer/go/publisher"
)

// TestEntity is a simple test struct for serialization testing
type TestEntity struct {
	ID   uint32 `json:"id" msgpack:"id"`
	Name string `json:"name" msgpack:"name"`
}

// MockPubCtx is a mock implementation of IPubCtx for testing
type MockPubCtx struct {
	publishFunc func(ctx context.Context, topic string, payload []byte) *errors.BrokerError
}

func (m *MockPubCtx) Publish(ctx context.Context, topic string, payload []byte) *errors.BrokerError {
	if m.publishFunc != nil {
		return m.publishFunc(ctx, topic, payload)
	}
	return nil
}

var _ = Describe("Publisher", func() {
	var mockCtx *MockPubCtx
	var ctx context.Context

	BeforeEach(func() {
		mockCtx = &MockPubCtx{}
		ctx = context.Background()
	})

	testPublish := func(name string, marshal func(any) ([]byte, error)) {
		entity := TestEntity{ID: 1, Name: fmt.Sprintf("Test Name: %v", name)}
		subject := fmt.Sprintf("test.subject.%v", name)

		expectedPayload, err := marshal(entity)
		Expect(err).NotTo(HaveOccurred())

		mockCtx.publishFunc = func(c context.Context, topic string, payload []byte) *errors.BrokerError {
			Expect(topic).To(Equal(subject))
			Expect(payload).To(Equal(expectedPayload))
			return nil
		}

		pub := publisher.NewPub[TestEntity](mockCtx, subject, marshal)
		err = pub.Publish(ctx, &entity)
		Expect(err).NotTo(HaveOccurred())
	}

	It("should publish with JSON format", func() {
		testPublish("JSON", json.Marshal)
	})

	It("should publish with MessagePack format", func() {
		testPublish("Msgpack", msgpack.Marshal)
	})

	It("should return error on publish failure", func() {
		entity := TestEntity{ID: 1, Name: "Test Name"}
		subject := "test.subject.error"
		testErr := fmt.Errorf("publish failed")

		mockCtx.publishFunc = func(c context.Context, topic string, payload []byte) *errors.BrokerError {
			return errors.NewBrokerError(testErr)
		}

		pub := publisher.NewPub[TestEntity](mockCtx, subject, json.Marshal)
		err := pub.Publish(ctx, &entity)
		Expect(err).To(HaveOccurred())
		Expect(err).To(BeAssignableToTypeOf(&errors.PubError{}))
	})
})

package publisher

import (
	"context"
	"encoding/json"
	"fmt"

	. "github.com/onsi/ginkgo/v2"
	. "github.com/onsi/gomega"
	"github.com/vmihailenco/msgpack/v5"

	"github.com/hiroaki-yamamoto/object-transfer/go/errors"
	"github.com/hiroaki-yamamoto/object-transfer/go/format"
)

// TestEntity is a simple test struct for serialization testing
type TestEntity struct {
	ID   uint32 `json:"id" msgpack:"id"`
	Name string `json:"name" msgpack:"name"`
}

// MockPubCtx is a mock implementation of IPubCtx for testing
type MockPubCtx struct {
	publishFunc func(ctx context.Context, topic string, payload []byte) error
}

func (m *MockPubCtx) Publish(ctx context.Context, topic string, payload []byte) error {
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

	testPublish := func(fmtType format.Format) {
		entity := TestEntity{ID: 1, Name: fmt.Sprintf("Test Name: %v", fmtType)}
		subject := fmt.Sprintf("test.subject.%v", fmtType)
		var expectedPayload []byte
		var err error

		switch fmtType {
		case format.FormatMsgpack:
			expectedPayload, err = msgpack.Marshal(entity)
		case format.FormatJSON:
			expectedPayload, err = json.Marshal(entity)
		}
		Expect(err).NotTo(HaveOccurred())

		mockCtx.publishFunc = func(c context.Context, topic string, payload []byte) error {
			Expect(topic).To(Equal(subject))
			Expect(payload).To(Equal(expectedPayload))
			return nil
		}

		pub := NewPub[TestEntity](mockCtx, subject, fmtType)
		err = pub.Publish(ctx, &entity)
		Expect(err).NotTo(HaveOccurred())
	}

	It("should publish with JSON format", func() {
		testPublish(format.FormatJSON)
	})

	It("should publish with MessagePack format", func() {
		testPublish(format.FormatMsgpack)
	})

	It("should return error on publish failure", func() {
		entity := TestEntity{ID: 1, Name: "Test Name"}
		subject := "test.subject.error"
		testErr := fmt.Errorf("publish failed")

		mockCtx.publishFunc = func(c context.Context, topic string, payload []byte) error {
			return testErr
		}

		pub := NewPub[TestEntity](mockCtx, subject, format.FormatJSON)
		err := pub.Publish(ctx, &entity)
		Expect(err).To(HaveOccurred())
		Expect(err).To(BeAssignableToTypeOf(&errors.PubError{}))
	})

	It("should handle unsupported format", func() {
		entity := TestEntity{ID: 1, Name: "Test Name"}
		subject := "test.subject.error"

		mockCtx.publishFunc = func(c context.Context, topic string, payload []byte) error {
			return nil
		}

		pub := NewPub[TestEntity](mockCtx, subject, format.Format("INVALID"))
		err := pub.Publish(ctx, &entity)
		Expect(err).To(HaveOccurred())
		Expect(err).To(BeAssignableToTypeOf(&errors.PubError{}))
	})
})

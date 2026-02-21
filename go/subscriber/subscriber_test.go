package subscriber_test

import (
	"context"
	"encoding/json"
	"fmt"

	. "github.com/onsi/ginkgo/v2"
	. "github.com/onsi/gomega"
	"github.com/vmihailenco/msgpack/v5"

	"github.com/hiroaki-yamamoto/object-transfer/go/format"
	"github.com/hiroaki-yamamoto/object-transfer/go/interfaces"
	"github.com/hiroaki-yamamoto/object-transfer/go/subscriber"
)

// TestEntity is a simple test struct for deserialization testing
type TestEntity struct {
	ID   uint32 `json:"id" msgpack:"id"`
	Name string `json:"name" msgpack:"name"`
}

// MockAck is a mock implementation of IAck
type MockAck struct {
	ackFunc func(ctx context.Context) error
	called  bool
}

func (m *MockAck) Ack(ctx context.Context) error {
	m.called = true
	if m.ackFunc != nil {
		return m.ackFunc(ctx)
	}
	return nil
}

// MockSubCtx is a mock implementation of ISubCtxTrait
type MockSubCtx struct {
	messages []interfaces.SubCtxMessage
	index    int
}

func (m *MockSubCtx) Subscribe(ctx context.Context) (<-chan interfaces.SubCtxMessage, error) {
	ch := make(chan interfaces.SubCtxMessage)
	go func() {
		defer close(ch)
		for _, msg := range m.messages {
			select {
			case <-ctx.Done():
				return
			case ch <- msg:
			}
		}
	}()
	return ch, nil
}

// MockSubOpt is a mock implementation of ISubOpt
type MockSubOpt struct {
	autoAck bool
	fmtType format.Format
}

func (m *MockSubOpt) GetAutoAck() bool {
	return m.autoAck
}

func (m *MockSubOpt) GetFormat() format.Format {
	return m.fmtType
}

// MockUnSub is a mock implementation of IUnSub
type MockUnSub struct {
	unsubFunc func(ctx context.Context) error
}

func (m *MockUnSub) Unsubscribe(ctx context.Context) error {
	if m.unsubFunc != nil {
		return m.unsubFunc(ctx)
	}
	return nil
}

var _ = Describe("Subscriber", func() {
	var ctx context.Context

	BeforeEach(func() {
		ctx = context.Background()
	})

	testSubscribe := func(fmtType format.Format, autoAck bool) {
		entities := []TestEntity{
			{ID: 1, Name: "Test1"},
			{ID: 2, Name: "Test2"},
			{ID: 3, Name: "Test3"},
		}

		var messages []interfaces.SubCtxMessage
		for _, entity := range entities {
			var payload []byte
			var err error

			switch fmtType {
			case format.FormatMsgpack:
				payload, err = msgpack.Marshal(entity)
			case format.FormatJSON:
				payload, err = json.Marshal(entity)
			}
			Expect(err).NotTo(HaveOccurred())

			ack := &MockAck{}
			messages = append(messages, interfaces.SubCtxMessage{
				Payload: payload,
				Ack:     ack,
			})
		}

		mockCtx := &MockSubCtx{messages: messages}
		mockOpt := &MockSubOpt{autoAck: autoAck, fmtType: fmtType}
		mockUnSub := &MockUnSub{}

		sub := subscriber.NewSub[TestEntity](mockCtx, mockUnSub, mockOpt)
		subMessages, err := sub.Subscribe(ctx)
		Expect(err).NotTo(HaveOccurred())

		var obtained []TestEntity
		for msg := range subMessages {
			Expect(msg.Error).To(BeNil())
			if msg.Item != nil {
				obtained = append(obtained, *msg.Item)
				if autoAck {
					// When auto-ack is enabled, ack should have been called
					Expect(msg.Ack).NotTo(BeNil())
				}
			}
		}

		Expect(obtained).To(Equal(entities))

		// Verify ack was called for each message if autoAck is true
		if autoAck {
			for _, msg := range messages {
				mockAck, ok := msg.Ack.(*MockAck)
				Expect(ok).To(BeTrue())
				Expect(mockAck.called).To(BeTrue())
			}
		}
	}

	It("should subscribe and deserialize with JSON format and auto-ack enabled", func() {
		testSubscribe(format.FormatJSON, true)
	})

	It("should subscribe and deserialize with MessagePack format and auto-ack enabled", func() {
		testSubscribe(format.FormatMsgpack, true)
	})

	It("should subscribe and deserialize with JSON format and auto-ack disabled", func() {
		testSubscribe(format.FormatJSON, false)
	})

	It("should subscribe and deserialize with MessagePack format and auto-ack disabled", func() {
		testSubscribe(format.FormatMsgpack, false)
	})

	It("should handle ack errors during auto-ack", func() {
		entity := TestEntity{ID: 1, Name: "Test"}
		payload, err := json.Marshal(entity)
		Expect(err).NotTo(HaveOccurred())

		ackErr := fmt.Errorf("ack failed")
		ack := &MockAck{
			ackFunc: func(ctx context.Context) error {
				return ackErr
			},
		}

		messages := []interfaces.SubCtxMessage{
			{Payload: payload, Ack: ack},
		}

		mockCtx := &MockSubCtx{messages: messages}
		mockOpt := &MockSubOpt{autoAck: true, fmtType: format.FormatJSON}
		mockUnSub := &MockUnSub{}

		sub := subscriber.NewSub[TestEntity](mockCtx, mockUnSub, mockOpt)
		subMessages, err := sub.Subscribe(ctx)
		Expect(err).NotTo(HaveOccurred())

		receivedErrors := 0
		for msg := range subMessages {
			if msg.Error != nil && msg.Item == nil {
				receivedErrors++
				Expect(msg.Error).To(Equal(ackErr))
			}
		}

		Expect(receivedErrors).To(Equal(1))
	})

	It("should handle JSON deserialization errors", func() {
		invalidPayload := []byte("invalid json")
		ack := &MockAck{}

		messages := []interfaces.SubCtxMessage{
			{Payload: invalidPayload, Ack: ack},
		}

		mockCtx := &MockSubCtx{messages: messages}
		mockOpt := &MockSubOpt{autoAck: false, fmtType: format.FormatJSON}
		mockUnSub := &MockUnSub{}

		sub := subscriber.NewSub[TestEntity](mockCtx, mockUnSub, mockOpt)
		subMessages, err := sub.Subscribe(ctx)
		Expect(err).NotTo(HaveOccurred())

		receivedErrors := 0
		for msg := range subMessages {
			if msg.Error != nil && msg.Item == nil {
				receivedErrors++
			}
		}

		Expect(receivedErrors).To(Equal(1))
	})

	It("should handle MessagePack deserialization errors", func() {
		invalidPayload := []byte{0xFF, 0xFE, 0xFD} // Invalid msgpack
		ack := &MockAck{}

		messages := []interfaces.SubCtxMessage{
			{Payload: invalidPayload, Ack: ack},
		}

		mockCtx := &MockSubCtx{messages: messages}
		mockOpt := &MockSubOpt{autoAck: false, fmtType: format.FormatMsgpack}
		mockUnSub := &MockUnSub{}

		sub := subscriber.NewSub[TestEntity](mockCtx, mockUnSub, mockOpt)
		subMessages, err := sub.Subscribe(ctx)
		Expect(err).NotTo(HaveOccurred())

		receivedErrors := 0
		for msg := range subMessages {
			if msg.Error != nil && msg.Item == nil {
				receivedErrors++
			}
		}

		Expect(receivedErrors).To(Equal(1))
	})

	It("should unsubscribe successfully", func() {
		unsubCalled := false
		mockUnSub := &MockUnSub{
			unsubFunc: func(ctx context.Context) error {
				unsubCalled = true
				return nil
			},
		}

		mockCtx := &MockSubCtx{}
		mockOpt := &MockSubOpt{}

		sub := subscriber.NewSub[TestEntity](mockCtx, mockUnSub, mockOpt)
		err := sub.Unsubscribe(ctx)

		Expect(err).NotTo(HaveOccurred())
		Expect(unsubCalled).To(BeTrue())
	})

	It("should propagate unsubscribe errors", func() {
		unsubErr := fmt.Errorf("unsubscribe failed")
		mockUnSub := &MockUnSub{
			unsubFunc: func(ctx context.Context) error {
				return unsubErr
			},
		}

		mockCtx := &MockSubCtx{}
		mockOpt := &MockSubOpt{}

		sub := subscriber.NewSub[TestEntity](mockCtx, mockUnSub, mockOpt)
		err := sub.Unsubscribe(ctx)

		Expect(err).To(Equal(unsubErr))
	})
})

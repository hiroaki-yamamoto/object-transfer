package noop

import "context"

// AckNoop is an acknowledgment handler that performs no operation.
//
// This can be used in scenarios where an acknowledgment callback is
// required by the interface but no actual acknowledgement should be sent
// to the message broker.
type AckNoop struct{}

// Ack performs no operation and always succeeds.
func (a *AckNoop) Ack(ctx context.Context) error {
	return nil
}

// NewAckNoop creates a new no-op acknowledgment handler.
func NewAckNoop() *AckNoop {
	return &AckNoop{}
}

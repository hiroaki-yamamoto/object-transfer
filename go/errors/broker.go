package errors

// BrokerError represents errors from broker-specific operations
// (for example, from NATS or JetStream) that provides a unified error
// interface for the messaging system.
type BrokerError struct {
	err error
}

// Error implements the error interface for BrokerError.
func (e *BrokerError) Error() string {
	return e.err.Error()
}

// Unwrap returns the underlying error for error chain inspection.
func (e *BrokerError) Unwrap() error {
	return e.err
}

// NewBrokerError creates a new BrokerError from any error type.
func NewBrokerError(err error) *BrokerError {
	return &BrokerError{err: err}
}

package errors

import "fmt"

// SubError represents errors that occur during subscription operations
// in the messaging system.
type SubError struct {
	Err error
}

// Error implements the error interface for SubError.
func (e *SubError) Error() string {
	if e.Err != nil {
		return fmt.Sprintf("Subscribe error: %v", e.Err)
	}
	return "Subscribe error"
}

// Unwrap returns the underlying error for error chain inspection.
func (e *SubError) Unwrap() error {
	return e.Err
}

// NewSubError creates a new SubError from any error.
func NewSubError(err error) *SubError {
	return &SubError{Err: err}
}

// SubBrokerError creates a new SubError wrapping a BrokerError.
func SubBrokerError(err *BrokerError) *SubError {
	return NewSubError(err)
}

// SubAckError creates a new SubError wrapping an AckError.
func SubAckError(err *AckError) *SubError {
	return NewSubError(err)
}

// SubJsonError creates a new SubError for JSON serialization/deserialization errors.
func SubJsonError(err error) *SubError {
	return NewSubError(fmt.Errorf("JSON error: %w", err))
}

// SubMessagePackDecodeError creates a new SubError for MessagePack decoding errors.
func SubMessagePackDecodeError(err error) *SubError {
	return NewSubError(fmt.Errorf("MessagePack decode error: %w", err))
}

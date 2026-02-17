package errors

import "fmt"

// PubError represents errors that occur during publishing operations
// in the messaging system.
type PubError struct {
	Err error
}

// Error implements the error interface for PubError.
func (e *PubError) Error() string {
	if e.Err != nil {
		return fmt.Sprintf("Publish error: %v", e.Err)
	}
	return "Publish error"
}

// Unwrap returns the underlying error for error chain inspection.
func (e *PubError) Unwrap() error {
	return e.Err
}

// NewPubError creates a new PubError from any error.
func NewPubError(err error) *PubError {
	return &PubError{Err: err}
}

// PubBrokerError creates a new PubError wrapping a BrokerError.
func PubBrokerError(err *BrokerError) *PubError {
	return NewPubError(err)
}

// PubJsonError creates a new PubError for JSON serialization/deserialization errors.
func PubJsonError(err error) *PubError {
	return NewPubError(fmt.Errorf("JSON error: %w", err))
}

// PubMessagePackEncodeError creates a new PubError for MessagePack encoding errors.
func PubMessagePackEncodeError(err error) *PubError {
	return NewPubError(fmt.Errorf("MessagePack encode error: %w", err))
}

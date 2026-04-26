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

// PubEncodeError creates a new PubError for encoding errors.
func PubEncodeError(err error) *PubError {
	return NewPubError(fmt.Errorf("Encode error: %w", err))
}

package errors

import "fmt"

// AckError represents errors that occur during acknowledgment operations
// in the messaging system.
type AckError struct {
	Err error
}

// Error implements the error interface for AckError.
func (e *AckError) Error() string {
	if e.Err != nil {
		return fmt.Sprintf("Acknowledgment error: %v", e.Err)
	}
	return "Acknowledgment error"
}

// Unwrap returns the underlying error for error chain inspection.
func (e *AckError) Unwrap() error {
	return e.Err
}

// NewAckError creates a new AckError from any error.
func NewAckError(err error) *AckError {
	return &AckError{Err: err}
}

// AckBrokerError creates a new AckError wrapping a BrokerError.
func AckBrokerError(err *BrokerError) *AckError {
	return NewAckError(err)
}

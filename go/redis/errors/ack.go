package errors

import "fmt"

// AckError represents errors that can occur during Redis acknowledgment operations.
type AckError struct {
	Err error
}

// Error implements the error interface for AckError.
func (e *AckError) Error() string {
	if e.Err != nil {
		return fmt.Sprintf("Redis acknowledgment error: %v", e.Err)
	}
	return "Redis acknowledgment error"
}

// Unwrap returns the underlying error for error chain inspection.
func (e *AckError) Unwrap() error {
	return e.Err
}

// NewAckError creates a new AckError from an error.
func NewAckError(err error) *AckError {
	return &AckError{
		Err: err,
	}
}

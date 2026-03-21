package errors

import "fmt"

// UnsubscribeError represents errors that can occur during Redis unsubscription operations.
type UnsubscribeError struct {
	Err error
}

// Error implements the error interface for UnsubscribeError.
func (e *UnsubscribeError) Error() string {
	if e.Err != nil {
		return fmt.Sprintf("Redis unsubscription error: %v", e.Err)
	}
	return "Redis unsubscription error"
}

// Unwrap returns the underlying error for error chain inspection.
func (e *UnsubscribeError) Unwrap() error {
	return e.Err
}

// NewUnsubscribeError creates a new UnsubscribeError from an error.
func NewUnsubscribeError(err error) *UnsubscribeError {
	return &UnsubscribeError{
		Err: err,
	}
}

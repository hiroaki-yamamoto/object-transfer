package errors

import "fmt"

// UnSubError represents errors that occur during unsubscribe operations
// in the messaging system.
type UnSubError struct {
	Err error
}

// Error implements the error interface for UnSubError.
func (e *UnSubError) Error() string {
	if e.Err != nil {
		return fmt.Sprintf("Unsubscribe error: %v", e.Err)
	}
	return "Unsubscribe error"
}

// Unwrap returns the underlying error for error chain inspection.
func (e *UnSubError) Unwrap() error {
	return e.Err
}

// NewUnSubError creates a new UnSubError from any error.
func NewUnSubError(err error) *UnSubError {
	return &UnSubError{Err: err}
}

// UnSubBrokerError creates a new UnSubError wrapping a BrokerError.
func UnSubBrokerError(err *BrokerError) *UnSubError {
	return NewUnSubError(err)
}

// ErrNoHandler is returned when no unsubscribe handler is found.
var ErrNoHandler = &UnSubError{Err: fmt.Errorf("no unsubscribe handler found")}

// NoHandlerError creates a new UnSubError for missing handler cases.
func NoHandlerError() *UnSubError {
	return &UnSubError{Err: fmt.Errorf("no unsubscribe handler found")}
}

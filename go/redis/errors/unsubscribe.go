package errors

import (
	"fmt"

	"github.com/redis/go-redis/v9"
)

// UnsubscribeError represents errors that can occur during Redis unsubscription operations.
type UnsubscribeError struct {
	Err *redis.Error
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
	if e.Err != nil {
		return *e.Err
	}
	return nil
}

// NewUnsubscribeError creates a new UnsubscribeError from a redis.Error.
func NewUnsubscribeError(err *redis.Error) *UnsubscribeError {
	return &UnsubscribeError{
		Err: err,
	}
}

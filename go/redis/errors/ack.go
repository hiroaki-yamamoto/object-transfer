package errors

import (
	"fmt"

	"github.com/redis/go-redis/v9"
)

// AckError represents errors that can occur during Redis acknowledgment operations.
type AckError struct {
	Err *redis.Error
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
	if e.Err != nil {
		return *e.Err
	}
	return nil
}

// NewAckError creates a new AckError from a redis.Error.
func NewAckError(err *redis.Error) *AckError {
	return &AckError{
		Err: err,
	}
}

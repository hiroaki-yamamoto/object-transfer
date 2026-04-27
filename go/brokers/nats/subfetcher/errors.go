package subfetcher

import (
	"fmt"
)

// NatsSubFetcherError represents errors during NATS SubFetcher operations.
type NatsSubFetcherError struct {
	cause error
}

// Error implements the error interface, formatting as
// "NATS SubFetcher error: <cause>".
func (e *NatsSubFetcherError) Error() string {
	return fmt.Sprintf("NATS SubFetcher error: %s", e.cause.Error())
}

// Unwrap returns the underlying error for error chain inspection.
func (e *NatsSubFetcherError) Unwrap() error {
	return e.cause
}

// NewSubFetcherError creates a new NatsSubFetcherError from any error type.
func NewSubFetcherError(err error) *NatsSubFetcherError {
	return &NatsSubFetcherError{cause: err}
}

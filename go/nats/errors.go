package nats

import (
	"fmt"

	errs "github.com/hiroaki-yamamoto/object-transfer/go/errors"
)

// NatsSubFetcherError represents errors during NATS SubFetcher operations.
type NatsSubFetcherError struct {
	cause error
}

// Error implements the error interface, formatting as
// "NATS JetStream Stream Creation Error: <cause>".
func (e *NatsSubFetcherError) Error() string {
	return fmt.Sprintf("NATS JetStream Stream Creation Error: %s", e.cause.Error())
}

// Unwrap returns the underlying error for error chain inspection.
func (e *NatsSubFetcherError) Unwrap() error {
	return e.cause
}

// NewJetStreamCreationError creates a [NatsSubFetcherError] for JetStream
// stream creation failures.  This corresponds to the
// NatsSubFetcherError::JetStreamStreamCreation variant in the Rust crate.
func NewJetStreamCreationError(err error) *NatsSubFetcherError {
	return &NatsSubFetcherError{cause: err}
}

// NewBrokerError converts any NATS-originated error into a [errs.BrokerError].
// This corresponds to the From<NatsKindError<T>> for BrokerError impl in the
// Rust crate.
func NewBrokerError(err error) *errs.BrokerError {
	return errs.NewBrokerError(err)
}

// Package noop provides a no-operation unsubscribe implementation.
//
// This package provides a no-op implementation of the unsubscribe interface,
// useful for scenarios where unsubscribe operations are not needed or
// should be skipped without performing any actual work.
package noop

import (
	"context"

	"github.com/hiroaki-yamamoto/object-transfer/go/errors"
)

// UnSubNoop is a no-operation unsubscribe handler.
//
// UnSubNoop is a simple implementation of IUnSub
// that performs no operations when unsubscribe is called.
//
// Depending on how it is configured, Unsubscribe will either:
// - Return nil without taking any action when ShouldErr is false
// - Return an error when ShouldErr is true
//
// This is useful for:
// - Default implementations where unsubscribe is not required
// - Testing and mocking scenarios
// - Cases where subscription cleanup is not necessary
type UnSubNoop struct {
	ShouldErr bool
}

// NewUnSubNoop creates a new instance of UnSubNoop.
func NewUnSubNoop(shouldErr bool) *UnSubNoop {
	return &UnSubNoop{ShouldErr: shouldErr}
}

// Unsubscribe performs a no-operation unsubscribe.
//
// Returns an error if ShouldErr is true,
// otherwise returns nil.
//
// Parameters:
// - ctx: context for cancellation and timeouts
//
// Returns an error if ShouldErr is true, otherwise nil.
func (u *UnSubNoop) Unsubscribe(ctx context.Context) error {
	if u.ShouldErr {
		return errors.ErrNoHandler
	}
	return nil
}

package errors

import "fmt"

// PublishError represents errors that can occur during Redis publish operations.
type PublishError struct {
	ErrType PublishErrorType
	Err     error
}

// PublishErrorType represents the type of publish error.
type PublishErrorType int

const (
	// GroupCreationError occurs when creating a consumer group in Redis fails.
	GroupCreationError PublishErrorType = iota
	// PushError occurs when pushing a message to a Redis stream fails.
	PushError
)

// Error implements the error interface for PublishError.
func (e *PublishError) Error() string {
	if e.Err != nil {
		switch e.ErrType {
		case GroupCreationError:
			return fmt.Sprintf("Group Creation Error: %v", e.Err)
		case PushError:
			return fmt.Sprintf("Message Pushing Error: %v", e.Err)
		default:
			return fmt.Sprintf("Redis publish error: %v", e.Err)
		}
	}
	return "Redis publish error"
}

// Unwrap returns the underlying error for error chain inspection.
func (e *PublishError) Unwrap() error {
	return e.Err
}

// NewGroupCreationError creates a PublishError for group creation failures.
func NewGroupCreationError(err error) *PublishError {
	return &PublishError{
		ErrType: GroupCreationError,
		Err:     err,
	}
}

// NewPushError creates a PublishError for message push failures.
func NewPushError(err error) *PublishError {
	return &PublishError{
		ErrType: PushError,
		Err:     err,
	}
}

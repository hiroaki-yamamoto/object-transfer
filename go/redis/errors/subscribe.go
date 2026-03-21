package errors

import "fmt"

// SubscribeError represents errors that can occur during Redis subscribe operations.
type SubscribeError struct {
	ErrType SubscribeErrorType
	Err     error
}

// SubscribeErrorType represents the type of subscribe error.
type SubscribeErrorType int

const (
	// SubscribeGroupCreationError occurs when creating a consumer group in Redis fails.
	SubscribeGroupCreationError SubscribeErrorType = iota
	// SubscribeAutoClaimError occurs when auto-claiming messages in Redis fails.
	SubscribeAutoClaimError
	// SubscribeReadError occurs when reading messages from a Redis stream fails.
	SubscribeReadError
	// SubscribeMissingDataFieldError occurs when a stream message is missing the required "data" field.
	SubscribeMissingDataFieldError
	// SubscribeInvalidDataTypeError occurs when a stream message's "data" field has an unexpected type.
	SubscribeInvalidDataTypeError
)

// Error implements the error interface for SubscribeError.
func (e *SubscribeError) Error() string {
	if e.Err != nil {
		switch e.ErrType {
		case SubscribeGroupCreationError:
			return fmt.Sprintf("Group Creation Error: %v", e.Err)
		case SubscribeAutoClaimError:
			return fmt.Sprintf("Auto-Claim Error: %v", e.Err)
		case SubscribeReadError:
			return fmt.Sprintf("Message Reading Error: %v", e.Err)
		case SubscribeMissingDataFieldError:
			return fmt.Sprintf("Missing Data Field Error: %v", e.Err)
		case SubscribeInvalidDataTypeError:
			return fmt.Sprintf("Invalid Data Type Error: %v", e.Err)
		default:
			return fmt.Sprintf("Redis subscribe error: %v", e.Err)
		}
	}
	return "Redis subscribe error"
}

// Unwrap returns the underlying error for error chain inspection.
func (e *SubscribeError) Unwrap() error {
	return e.Err
}

// NewSubscribeGroupCreationError creates a SubscribeError for group creation failures.
func NewSubscribeGroupCreationError(err error) *SubscribeError {
	return &SubscribeError{
		ErrType: SubscribeGroupCreationError,
		Err:     err,
	}
}

// NewSubscribeAutoClaimError creates a SubscribeError for auto-claim failures.
func NewSubscribeAutoClaimError(err error) *SubscribeError {
	return &SubscribeError{
		ErrType: SubscribeAutoClaimError,
		Err:     err,
	}
}

// NewSubscribeReadError creates a SubscribeError for message reading failures.
func NewSubscribeReadError(err error) *SubscribeError {
	return &SubscribeError{
		ErrType: SubscribeReadError,
		Err:     err,
	}
}

// NewSubscribeMissingDataFieldError creates a SubscribeError for messages missing the "data" field.
func NewSubscribeMissingDataFieldError(msgID string) *SubscribeError {
	return &SubscribeError{
		ErrType: SubscribeMissingDataFieldError,
		Err:     fmt.Errorf("message %q is missing the required \"data\" field", msgID),
	}
}

// NewSubscribeInvalidDataTypeError creates a SubscribeError for messages with an unexpected "data" field type.
func NewSubscribeInvalidDataTypeError(msgID string, got interface{}) *SubscribeError {
	return &SubscribeError{
		ErrType: SubscribeInvalidDataTypeError,
		Err:     fmt.Errorf("message %q has unexpected \"data\" field type %T", msgID, got),
	}
}

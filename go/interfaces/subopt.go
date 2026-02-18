package interfaces

import "github.com/hiroaki-yamamoto/object-transfer/go/format"

// ISubOpt provides options that influence subscription behavior such as auto-ack and format.
type ISubOpt interface {
	// GetAutoAck returns whether automatic acknowledgment is enabled.
	// When true, messages are automatically acknowledged upon receipt.
	GetAutoAck() bool

	// GetFormat returns the serialization format used for messages.
	GetFormat() format.Format
}

package interfaces

// Format represents the serialization format used for messages.
type Format string

const (
	// FormatJSON represents JSON serialization format
	FormatJSON Format = "json"
	// FormatMsgpack represents MessagePack serialization format
	FormatMsgpack Format = "msgpack"
)

// ISubOpt provides options that influence subscription behavior such as auto-ack and format.
type ISubOpt interface {
	// GetAutoAck returns whether automatic acknowledgment is enabled.
	// When true, messages are automatically acknowledged upon receipt.
	GetAutoAck() bool

	// GetFormat returns the serialization format used for messages.
	GetFormat() Format
}

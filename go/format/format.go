package format

// Format represents the serialization format used for messages.
type Format string

const (
	// FormatJSON represents JSON serialization format
	FormatJSON Format = "JSON"
	// FormatMsgpack represents MessagePack serialization format
	FormatMsgpack Format = "MessagePack"
)

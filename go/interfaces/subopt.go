package interfaces

// ISubOpt provides options that influence subscription behavior such as auto-ack and unmarshal func.
type ISubOpt interface {
	// GetAutoAck returns whether automatic acknowledgment is enabled.
	// When true, messages are automatically acknowledged upon receipt.
	GetAutoAck() bool

	// GetUnmarshalFunc returns the function used to deserialize messages.
	GetUnmarshalFunc() func([]byte, any) error
}

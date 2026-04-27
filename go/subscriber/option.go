package subscriber

// Option configures the behavior of a [Sub] subscriber.
// The zero value is not valid; use [NewOption] to construct an instance.
type Option struct {
	autoAck bool
}

// NewOption returns a new Option with default settings.
// By default, auto-acknowledgment is enabled (autoAck: true), meaning each
// received message is automatically acknowledged before it is delivered to
// the caller.
func NewOption() *Option {
	return &Option{
		autoAck: true,
	}
}

// AutoAck sets whether messages should be acknowledged automatically.
// When autoAck is true, the subscriber acknowledges each message immediately
// after it is received from the broker, before delivering it to the caller.
// When autoAck is false, the caller is responsible for calling Ack on each
// [SubMessage] to signal successful processing.
// Returns the Option to support method chaining.
func (o *Option) AutoAck(autoAck bool) *Option {
	o.autoAck = autoAck
	return o
}

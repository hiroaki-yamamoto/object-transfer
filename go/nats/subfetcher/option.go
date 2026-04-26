package subfetcher

import (
	"github.com/nats-io/nats.go"
)

// AckSubOptions provides configuration options for creating an acknowledgment-based subscriber.
//
// This struct provides a builder pattern for configuring NATS JetStream
// consumers with pull-based message consumption and automatic acknowledgment options.
type AckSubOptions struct {
	streamConfig   *nats.StreamConfig
	consumerConfig *nats.ConsumerConfig
}

// NewAckSubOptions creates a new AckSubOptions with the specified name.
//
// Arguments:
//   - name: The name for both the stream and consumer
//
// Returns:
// A new AckSubOptions instance with default settings
func NewAckSubOptions(name string) *AckSubOptions {
	return &AckSubOptions{
		streamConfig: &nats.StreamConfig{
			Name: name,
		},
		consumerConfig: &nats.ConsumerConfig{
			Durable: name,
		},
	}
}

// Name sets the stream name.
//
// Arguments:
//   - name: The name to assign to the stream
//
// Returns:
// The AckSubOptions instance for method chaining
func (o *AckSubOptions) Name(name string) *AckSubOptions {
	o.streamConfig.Name = name
	return o
}

// Subjects sets the subjects that the stream should listen to.
//
// Arguments:
//   - subjects: One or more subject patterns to subscribe to (variadic)
//
// Returns:
// The AckSubOptions instance for method chaining
func (o *AckSubOptions) Subjects(subjects ...string) *AckSubOptions {
	o.streamConfig.Subjects = subjects
	return o
}

// DurableName sets the durable name for the consumer.
//
// A durable consumer will persist its state and can resume consumption
// after disconnection.
//
// Arguments:
//   - durableName: The durable name for the consumer
//
// Returns:
// The AckSubOptions instance for method chaining
func (o *AckSubOptions) DurableName(durableName string) *AckSubOptions {
	o.consumerConfig.Durable = durableName
	return o
}

// StreamConfig sets the complete stream configuration.
//
// This replaces the entire stream configuration with the provided one.
//
// Arguments:
//   - streamConfig: The stream configuration to use
//
// Returns:
// The AckSubOptions instance for method chaining
func (o *AckSubOptions) StreamConfig(streamConfig *nats.StreamConfig) *AckSubOptions {
	o.streamConfig = streamConfig
	return o
}

// ConsumerConfig sets the complete consumer configuration.
//
// This replaces the entire consumer configuration with the provided one.
//
// Arguments:
//   - consumerConfig: The consumer configuration to use
//
// Returns:
// The AckSubOptions instance for method chaining
func (o *AckSubOptions) ConsumerConfig(consumerConfig *nats.ConsumerConfig) *AckSubOptions {
	o.consumerConfig = consumerConfig
	return o
}

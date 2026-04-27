package config

// PublisherConfig holds the configuration settings needed to initialize and manage
// a Redis publisher instance, including the consumer group name for Redis streams.
type PublisherConfig struct {
	// GroupName is the Redis stream consumer group name used by the publisher.
	// If not set, it defaults to the stream name.
	GroupName *string
	// StreamLength is the maximum length of the Redis stream.
	// Defaults to 500,000 messages.
	StreamLength uint64
}

const maxStreamLengthDefault = 500_000

// NewPublisherConfig creates a new PublisherConfig instance with default values.
//
// Default values:
//   - GroupName: nil (defaults to the stream name)
//   - StreamLength: 500,000 messages
func NewPublisherConfig() *PublisherConfig {
	return &PublisherConfig{
		GroupName:    nil,
		StreamLength: maxStreamLengthDefault,
	}
}

// WithGroupName sets the Redis stream consumer group name and returns the updated config.
// This method supports method chaining.
func (cfg *PublisherConfig) WithGroupName(groupName string) *PublisherConfig {
	cfg.GroupName = &groupName
	return cfg
}

// WithStreamLength sets the maximum length of the Redis stream and returns the updated config.
// This method supports method chaining.
func (cfg *PublisherConfig) WithStreamLength(length uint64) *PublisherConfig {
	cfg.StreamLength = length
	return cfg
}

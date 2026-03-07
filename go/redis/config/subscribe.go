package config

// SubscriberConfig holds the configuration settings needed to initialize and manage
// a Redis stream subscriber, including fetch count and block time.
//
// Default values:
//   - ConsumerName: same as TopicName
//   - GroupName: same as TopicName
//   - NumFetch: 10
//   - BlockTime: 5000 ms (5 seconds)
//   - AutoClaim: 30000 ms (min-idle-time for XAUTOCLAIM)
type SubscriberConfig struct {
	// ConsumerName is the name of the consumer within the consumer group.
	// If not set, it defaults to the stream name.
	ConsumerName string
	// GroupName is the Redis stream consumer group name.
	// If not set, it defaults to the stream name.
	GroupName string
	// TopicName is the name of the Redis stream topic.
	TopicName string
	// NumFetch is the number of messages to fetch per read.
	// Defaults to 10.
	NumFetch uint64
	// BlockTime is the blocking time in milliseconds for each read.
	// Defaults to 5000 (5 seconds).
	BlockTime uint64
	// AutoClaim is the minimum idle time in milliseconds for auto-claiming pending messages.
	// If set to 0, auto-claiming is disabled.
	// Defaults to 30000 (30 seconds).
	AutoClaim uint64
}

const (
	defaultNumFetch  = 10
	defaultBlockTime = 5000
	defaultAutoClaim = 30000
)

// NewSubscriberConfig creates a new SubscriberConfig instance with defaults and the given topic name.
//
// Default values:
//   - ConsumerName: same as topicName
//   - GroupName: same as topicName
//   - NumFetch: 10
//   - BlockTime: 5000 ms (5 seconds)
//   - AutoClaim: 30000 ms (30 seconds, min-idle-time for XAUTOCLAIM)
func NewSubscriberConfig(topicName string) *SubscriberConfig {
	return &SubscriberConfig{
		ConsumerName: topicName,
		GroupName:    topicName,
		TopicName:    topicName,
		NumFetch:     defaultNumFetch,
		BlockTime:    defaultBlockTime,
		AutoClaim:    defaultAutoClaim,
	}
}

// WithConsumerName sets the consumer name and returns the updated config.
// This method supports method chaining.
func (cfg *SubscriberConfig) WithConsumerName(consumerName string) *SubscriberConfig {
	cfg.ConsumerName = consumerName
	return cfg
}

// WithGroupName sets the consumer group name and returns the updated config.
// This method supports method chaining.
func (cfg *SubscriberConfig) WithGroupName(groupName string) *SubscriberConfig {
	cfg.GroupName = groupName
	return cfg
}

// WithTopicName sets the topic (stream) name and returns the updated config.
// This method supports method chaining.
func (cfg *SubscriberConfig) WithTopicName(topicName string) *SubscriberConfig {
	cfg.TopicName = topicName
	return cfg
}

// WithNumFetch sets how many messages to fetch per read and returns the updated config.
// This method supports method chaining.
func (cfg *SubscriberConfig) WithNumFetch(count uint64) *SubscriberConfig {
	cfg.NumFetch = count
	return cfg
}

// WithBlockTime sets the blocking time in milliseconds for each read and returns the updated config.
// This method supports method chaining.
func (cfg *SubscriberConfig) WithBlockTime(millis uint64) *SubscriberConfig {
	cfg.BlockTime = millis
	return cfg
}

// WithAutoClaim sets the minimum idle time in milliseconds for auto-claiming pending messages
// and returns the updated config. If the value is 0, auto-claiming is disabled.
// This method supports method chaining.
func (cfg *SubscriberConfig) WithAutoClaim(millis uint64) *SubscriberConfig {
	cfg.AutoClaim = millis
	return cfg
}

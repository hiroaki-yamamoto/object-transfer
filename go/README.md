# Object Transfer - Go Binding

| Service | Status |
|---------|--------|
| Go Test | [![GoTestImg]][GoTest] |

A type-safe, pluggable message publisher and subscriber library for Go. Supports multiple backends (NATS JetStream, Redis) and serialization formats (JSON, MessagePack).

[GoTestImg]: https://github.com/hiroaki-yamamoto/object-transfer/actions/workflows/test_go.yml/badge.svg
[GoTest]: https://github.com/hiroaki-yamamoto/object-transfer/actions/workflows/test_go.yml

## Features

- **Type-safe generics**: Strongly-typed publishers and subscribers with compile-time safety
- **Multiple backends**: Support for NATS JetStream and Redis out of the box
- **Flexible serialization**: JSON and MessagePack format support
- **Acknowledgment handling**: Manual and automatic message acknowledgment
- **Context support**: Built-in support for cancellation and timeouts

## Installation

```bash
go get github.com/hiroaki-yamamoto/object-transfer/go
```

## Quick Start

### NATS JetStream Example

#### Publisher

```go
package main

import (
	"context"
	"log"

	nats "github.com/nats-io/nats.go"

	otNats "github.com/hiroaki-yamamoto/object-transfer/go/brokers/nats"
	"github.com/hiroaki-yamamoto/object-transfer/go/publisher"
	"encoding/json"
)

type UserCreated struct {
	ID   uint64 `json:"id" msgpack:"id"`
	Name string `json:"name" msgpack:"name"`
}

func main() {
	ctx := context.Background()

	// Connect to NATS
	conn, err := nats.Connect("nats://localhost:4222")
	if err != nil {
		log.Fatal(err)
	}
	defer conn.Close()

	// Get JetStream context
	js, err := conn.JetStream()
	if err != nil {
		log.Fatal(err)
	}

	// Create a publisher for the user.created topic
	pubCtx := otNats.NewPubCtx(js)
	pub := publisher.NewPub[UserCreated](pubCtx, "user.created", json.Marshal)

	// Publish an event
	event := UserCreated{ID: 42, Name: "Jane Doe"}
	err = pub.Publish(ctx, &event)
	if err != nil {
		log.Printf("Failed to publish: %v\n", err)
	}
}
```

#### Subscriber

```go
package main

import (
	"context"
	"log"

	nats "github.com/nats-io/nats.go"

	"github.com/hiroaki-yamamoto/object-transfer/go/brokers/nats/subfetcher"
	"github.com/hiroaki-yamamoto/object-transfer/go/subscriber"
	"encoding/json"
)

type UserCreated struct {
	ID   uint64 `json:"id" msgpack:"id"`
	Name string `json:"name" msgpack:"name"`
}

func main() {
	ctx := context.Background()

	// Connect to NATS
	conn, err := nats.Connect("nats://localhost:4222")
	if err != nil {
		log.Fatal(err)
	}
	defer conn.Close()

	// Get JetStream context
	js, err := conn.JetStream()
	if err != nil {
		log.Fatal(err)
	}

	// Create subscription options
	opts := subfetcher.NewAckSubOptions("user-events").
		Subjects("user.created").
		DurableName("user-created-consumer")

	// Create sub-fetcher
	fetcher, err := subfetcher.NewSubFetcher(ctx, js, opts)
	if err != nil {
		log.Fatal(err)
	}

	// Create subscriber options (auto-ack disabled for manual acknowledgment)
	subOpt := subscriber.NewOption().AutoAck(false)

	// Create subscriber
	sub, subErr := subscriber.NewSub[UserCreated](fetcher, json.Unmarshal, fetcher, subOpt)
	if subErr != nil {
		log.Fatal(subErr)
	}

	// Subscribe to messages
	messages, subErr := sub.Subscribe(ctx)
	if subErr != nil {
		log.Fatal(subErr)
	}

	// Process messages
	for msg := range messages {
		if msg.Error != nil {
			log.Printf("Error: %v\n", msg.Error)
			continue
		}

		if msg.Item != nil {
			log.Printf("Received: %+v\n", *msg.Item)
			// Acknowledge the message
			msg.Ack.Ack(ctx)
		}
	}
}
```

### Redis Example

#### Publisher

```go
package main

import (
	"context"
	"log"

	goredis "github.com/redis/go-redis/v9"

	"github.com/hiroaki-yamamoto/object-transfer/go/publisher"
	redisPublisher "github.com/hiroaki-yamamoto/object-transfer/go/brokers/redis/publisher"
	redisConfig "github.com/hiroaki-yamamoto/object-transfer/go/brokers/redis/config"
	"encoding/json"
)

type UserCreated struct {
	ID   uint64 `json:"id" msgpack:"id"`
	Name string `json:"name" msgpack:"name"`
}

func main() {
	ctx := context.Background()

	// Connect to Redis
	client := goredis.NewClient(&goredis.Options{
		Addr: "localhost:6379",
	})
	defer client.Close()

	// Create publisher configuration
	cfg := redisConfig.NewPublisherConfig().
		WithGroupName("user-service").
		WithStreamLength(1000)

	// Create Redis publisher context
	rpub := redisPublisher.New(client, cfg)

	// Create a publisher for the user.created stream
	pub := publisher.NewPub[UserCreated](rpub, "user.created", json.Marshal)

	// Publish an event
	event := UserCreated{ID: 42, Name: "Jane Doe"}
	err := pub.Publish(ctx, &event)
	if err != nil {
		log.Printf("Failed to publish: %v\n", err)
	}
}
```

#### Subscriber

```go
package main

import (
	"context"
	"log"

	goredis "github.com/redis/go-redis/v9"

	redisSubscriber "github.com/hiroaki-yamamoto/object-transfer/go/brokers/redis/subscriber"
	redisConfig "github.com/hiroaki-yamamoto/object-transfer/go/brokers/redis/config"
	"github.com/hiroaki-yamamoto/object-transfer/go/subscriber"
	"encoding/json"
)

type UserCreated struct {
	ID   uint64 `json:"id" msgpack:"id"`
	Name string `json:"name" msgpack:"name"`
}

func main() {
	ctx := context.Background()

	// Connect to Redis
	client := goredis.NewClient(&goredis.Options{
		Addr: "localhost:6379",
	})
	defer client.Close()

	// Create subscription configuration
	cfg := redisConfig.NewSubscriberConfig("user.created").
		WithGroupName("user-service").
		WithConsumerName("user-service-1")

	// Create Redis subscriber
	rsub := redisSubscriber.New(client, cfg)

	// Create subscriber
	sub, subErr := subscriber.NewSub[UserCreated](rsub, json.Unmarshal, rsub, subscriber.NewOption())
	if subErr != nil {
		log.Fatal(subErr)
	}

	// Subscribe to messages
	messages, subErr := sub.Subscribe(ctx)
	if subErr != nil {
		log.Fatal(subErr)
	}

	// Process messages
	for msg := range messages {
		if msg.Error != nil {
			log.Printf("Error: %v\n", msg.Error)
			continue
		}

		if msg.Item != nil {
			log.Printf("Received: %+v\n", *msg.Item)
			// Acknowledge the message
			msg.Ack.Ack(ctx)
		}
	}
}
```

## Serialization Formats

The library supports two serialization formats:

### JSON

```go
pub := publisher.NewPub[MyType](ctx, "topic", json.Marshal)
```

### MessagePack

```go
pub := publisher.NewPub[MyType](ctx, "topic", msgpack.Marshal)
```

## Error Handling

Both publishers and subscribers return structured errors:

```go
if err := pub.Publish(ctx, &event); err != nil {
	// err is of type *errors.PubError
	log.Printf("Failed to publish: %v\n", err)
}

messages, err := sub.Subscribe(ctx)
if err != nil {
	// err is of type *errors.SubError
	log.Printf("Failed to subscribe: %v\n", err)
}
```

## Message Acknowledgment

### Automatic Acknowledgment (NATS)

```go
subOpt := subscriber.NewOption().AutoAck(true) // Messages are acked automatically
```

### Manual Acknowledgment

```go
subOpt := subscriber.NewOption().AutoAck(false) // You control when to ack

for msg := range messages {
	if msg.Item != nil {
		// Process message
		msg.Ack.Ack(ctx) // Acknowledge when ready
	}
}
```

## Context Usage

Both publishers and subscribers respect Go context for cancellation:

```go
// With timeout
ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
defer cancel()

err := pub.Publish(ctx, &event)

messages, err := sub.Subscribe(ctx)
```

## Common Patterns

### Batch Publishing

```go
events := []UserCreated{
	{ID: 1, Name: "Alice"},
	{ID: 2, Name: "Bob"},
	{ID: 3, Name: "Charlie"},
}

for _, event := range events {
	if err := pub.Publish(ctx, &event); err != nil {
		log.Printf("Failed to publish: %v\n", err)
	}
}
```

### Graceful Shutdown

```go
// Create a cancellable context
ctx, cancel := context.WithCancel(context.Background())
defer cancel()

messages, _ := sub.Subscribe(ctx)

// On shutdown signal
go func() {
	<-shutdownSignal
	cancel() // This will stop the subscription
}()

for msg := range messages {
	// Process message
}
```

## Backends

### NATS JetStream

Provides durable, scalable message streaming with consumer groups and acknowledgment tracking.

- Packages:
  - `github.com/hiroaki-yamamoto/object-transfer/go/brokers/nats`
  - `github.com/hiroaki-yamamoto/object-transfer/go/brokers/nats/subfetcher`
- Dependencies: `github.com/nats-io/nats.go`

### Redis

Uses Redis Streams for durable message storage with consumer group support.

- Packages:
  - `github.com/hiroaki-yamamoto/object-transfer/go/brokers/redis/publisher`
  - `github.com/hiroaki-yamamoto/object-transfer/go/brokers/redis/subscriber`
- Dependencies: `github.com/redis/go-redis/v9`

## Testing

Each backend includes comprehensive test suites. Run tests with:

```bash
make test
```

See individual test files for integration test examples.

## License

See [LICENSE.md](../LICENSE.md) for details.

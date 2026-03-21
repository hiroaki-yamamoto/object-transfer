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

	"github.com/hiroaki-yamamoto/object-transfer/go/format"
	"github.com/hiroaki-yamamoto/object-transfer/go/nats"
	"github.com/hiroaki-yamamoto/object-transfer/go/publisher"
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
	pubCtx := nats.NewPubCtx(js)
	pub := publisher.NewPub[UserCreated](pubCtx, "user.created", format.FormatJSON)

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

	"github.com/hiroaki-yamamoto/object-transfer/go/format"
	otNats "github.com/hiroaki-yamamoto/object-transfer/go/nats"
	"github.com/hiroaki-yamamoto/object-transfer/go/subscriber"
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
	options := otNats.NewAckSubOptions(format.FormatJSON, "user-events")
	options.Subjects("user.created")
	options.DurableName("user-created-consumer")
	options.AutoAck(false) // Manual acknowledgment

	// Create sub-fetcher
	fetcher, err := otNats.NewSubFetcher(js, options)
	if err != nil {
		log.Fatal(err)
	}

	// Create subscriber
	sub := subscriber.NewSub[UserCreated](fetcher, fetcher, options)

	// Subscribe to messages
	messages, err := sub.Subscribe(ctx)
	if err != nil {
		log.Fatal(err)
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

	"github.com/hiroaki-yamamoto/object-transfer/go/format"
	"github.com/hiroaki-yamamoto/object-transfer/go/publisher"
	redisPublisher "github.com/hiroaki-yamamoto/object-transfer/go/redis/publisher"
	redisConfig "github.com/hiroaki-yamamoto/object-transfer/go/redis/config"
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
	cfg := &redisConfig.PublisherConfig{
		GroupName:    "user-service",
		StreamLength: 1000,
	}

	// Create Redis publisher context
	rpub := redisPublisher.New(client, cfg)

	// Create a publisher for the user.created stream
	pub := publisher.NewPub[UserCreated](rpub, "user.created", format.FormatJSON)

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

	"github.com/hiroaki-yamamoto/object-transfer/go/format"
	redisSubscriber "github.com/hiroaki-yamamoto/object-transfer/go/redis/subscriber"
	redisConfig "github.com/hiroaki-yamamoto/object-transfer/go/redis/config"
	"github.com/hiroaki-yamamoto/object-transfer/go/subscriber"
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
	cfg := &redisConfig.SubscriberConfig{
		GroupName:    "user-service",
		ConsumerName: "user-service-1",
	}

	// Create Redis subscriber
	rsub := redisSubscriber.New(client, "user.created", format.FormatJSON, cfg)

	// Create subscriber
	sub := subscriber.NewSub[UserCreated](rsub, rsub, rsub)

	// Subscribe to messages
	messages, err := sub.Subscribe(ctx)
	if err != nil {
		log.Fatal(err)
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
pub := publisher.NewPub[MyType](ctx, "topic", format.FormatJSON)
```

### MessagePack

```go
pub := publisher.NewPub[MyType](ctx, "topic", format.FormatMsgpack)
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
options := nats.NewAckSubOptions(format.FormatJSON, "events")
options.AutoAck(true) // Messages are acked automatically
```

### Manual Acknowledgment

```go
options := nats.NewAckSubOptions(format.FormatJSON, "events")
options.AutoAck(false) // You control when to ack

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

- Package: `github.com/hiroaki-yamamoto/object-transfer/go/nats`
- Dependencies: `github.com/nats-io/nats.go`

### Redis

Uses Redis Streams for durable message storage with consumer group support.

- Packages:
  - `github.com/hiroaki-yamamoto/object-transfer/go/redis/publisher`
  - `github.com/hiroaki-yamamoto/object-transfer/go/redis/subscriber`
- Dependencies: `github.com/redis/go-redis/v9`

## Testing

Each backend includes comprehensive test suites. Run tests with:

```bash
make test
```

See individual test files for integration test examples.

## License

See [LICENSE.md](../LICENSE.md) for details.

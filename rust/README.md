# Object Transfer Library for Rust

## CI/CD Status

| Service | Status |
|---------|--------|
| Crates.io | [![Crates.io Version Img]][Crates.io] |
| Code Test | [![Test Rust Code Img]][Test Rust Code] |

[Test Rust Code Img]: https://github.com/hiroaki-yamamoto/object-transfer/actions/workflows/test_rust.yml/badge.svg
[Test Rust Code]: https://github.com/hiroaki-yamamoto/object-transfer/actions/workflows/test_rust.yml
[Crates.io Version Img]: https://img.shields.io/crates/v/object_transfer
[Crates.io]: https://crates.io/crates/object_transfer


## Description

This library provides a simple and efficient way to transfer objects between
different parts of an application or between different applications through
message brokers like NATS.
It supports serialization and/or deserialization of various data formats, making
it easy to send and/or receive complex data structures.

## Installation

If you have cargo-edit installed, you can add this library to your project
by running:

```sh
cargo add object_transfer
```

Alternatively, you can manually add the following line to your `Cargo.toml` file:

```toml
[dependencies]
object_transfer = "x.x.x"
```

where `x.x.x` is the desired version of the library. For the latst version,
please check [Crates.io](https://crates.io/crates/object_transfer).

## Example of Use

```rust
// Import required traits and error types from the object_transfer library
use ::std::sync::Arc;
use ::object_transfer::traits::{PubTrait, SubTrait};
use ::object_transfer::error::{PubError, SubError};
use ::serde::{Serialize, Deserialize};

// Define a data structure that will be transmitted via message broker
#[derive(Serialize, Deserialize, Debug)]
pub struct MyData {
  pub id: u32,
  pub name: String,
}

// Example function demonstrating how to publish data
async fn publish_example(
  publisher: Arc<dyn PubTrait + Send + Sync>,
) -> Result<MyData, PubError> {
  let data = MyData { id: 1, name: "Example".to_string() };
  publisher.publish(&data).await.unwrap();
}

// Example function demonstrating how to subscribe and receive data
async fn subscribe_example(
  subscriber: Arc<dyn SubTrait<Item = MyData> + Send + Sync>,
) -> Result<MyData, SubError> {
  let mut stream = subscriber.subscribe().await.unwrap();
  // Process incoming messages from the stream
  while let Some(data) = stream.next().await {
    match data {
      Ok(my_data) => println!("Received data: {:?}", my_data),
      Err(e) => eprintln!("Error receiving data: {}", e),
    }
  }
}

// Main entry point with tokio async runtime
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Connect to NATS server
  let client = async_nats::connect("demo.nats.io").await?;
  let js = Arc::new(async_nats::jetstream::new(client));

  // Configure subscription options with acknowledgment settings
  let options = Arc::new(
    AckSubOptions::new(Format::JSON, Arc::from("events"))
      .subjects(vec!["mydata.created"])
      .durable_name("user-created")
      .auto_ack(false), // Require manual acknowledgment
  );

  // Initialize subscriber with fetcher for retrieving messages
  let fetcher = Arc::new(SubFetcher::new(js.clone(), options.clone()).await?);
  let unsub = Some(fetcher.clone() as Arc<dyn UnSubTrait + Send + Sync>);
  let subscriber: Sub<MyData> = Sub::new(fetcher, unsub, options);

  // Initialize publisher for sending messages to a specific subject
  let publisher: Pub<MyData> = Pub::new(
    js.clone(),
    "mydata.created",
    Format::JSON,
  );

  // Create and publish a sample event
  let event = MyData {
    id: 42,
    name: "Jane Doe".to_string(),
  };
  publisher.publish(&event).await?;

  // Subscribe to messages and wait for incoming data
  let mut stream = subscriber.subscribe().await?;

  // Process the received message with manual acknowledgment
  if let Some(Ok((event, ack))) = stream.next().await {
    println!("received {:?}", event);
    // Manually acknowledge the message since auto_ack is disabled
    ack.ack().await?;
  } else {
    println!("no event received");
  }
}
```

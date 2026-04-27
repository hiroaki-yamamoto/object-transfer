# Object Transfer Library for Rust

## CI/CD Status

| Service | Status |
|---------|--------|
| Crates.io | [![Crates.io Version Img]][Crates.io] |
| Rust Test | [![Test Rust Code Img]][Test Rust Code] |

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

## Example of Use with JSON Format

```rust
use std::sync::Arc;
use futures::StreamExt;
use serde::{Serialize, Deserialize};
use object_transfer::{
  encoders::JSONEncoder,
  encoders::JSONDecoder,
  Pub, Sub,
  SubOpt,
  traits::{PubTrait, SubTrait},
};
use object_transfer::brokers::nats::{SubFetcherOpt, SubFetcher};

// Define a data structure that will be transmitted via message broker
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MyData {
  pub id: u32,
  pub name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Connect to NATS server
  let client = async_nats::connect("demo.nats.io").await?;
  let js = Arc::new(async_nats::jetstream::new(client));

  // Initialize publisher with JSON encoder
  let publisher: Pub<MyData, _> = Pub::new(
    js.clone(),
    "mydata.created",
    Arc::new(JSONEncoder::new()),
  );

  // Create and publish a sample event
  let event = MyData {
    id: 42,
    name: "Jane Doe".to_string(),
  };
  publisher.publish(&event).await?;

  // Configure subscription with acknowledgment settings
  let fetcher_opt = SubFetcherOpt::new(Arc::from("events"))
    .subjects(vec!["mydata.created"])
    .durable_name("user-created");

  // Initialize subscriber with JSON decoder
  let fetcher = Arc::new(SubFetcher::new(js.clone(), fetcher_opt).await?);
  let unsub = fetcher.clone();
  let options = SubOpt::new().auto_ack(false); // Require manual acknowledgment
  let subscriber: Sub<MyData, _> = Sub::new(fetcher, unsub, Arc::new(JSONDecoder::new()), options);

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

  Ok(())
}
```

## Custom Serialization Formats

The library supports any serialization format by implementing the [`Encoder`](src/encoders/traits.rs) and [`Decoder`](src/encoders/traits.rs) traits.
This "any-format" design allows you to use JSON, MessagePack, Protocol Buffers, CBOR, or any custom format without modifying the library.

### Implementing a Custom Encoder and Decoder

Here's an example of implementing a simple plain-text encoder:

```rust
use bytes::Bytes;
use serde::Serialize;
use object_transfer::encoders::Encoder;

#[derive(Serialize)]
struct MyData {
    id: u32,
    name: String,
}

// Custom plain-text encoder
struct PlainTextEncoder;

impl Encoder for PlainTextEncoder {
    type Item = MyData;
    type Error = std::fmt::Error;

    fn encode(&self, item: &Self::Item) -> Result<Bytes, Self::Error> {
        let text = format!("id={},name={}", item.id, item.name);
        Ok(Bytes::from(text))
    }
}
```

Implement the corresponding decoder:

```rust
use bytes::Bytes;
use serde::de::DeserializeOwned;
use object_transfer::encoders::Decoder;
use std::num::ParseIntError;

#[derive(Debug)]
enum ParseError {
    InvalidFormat,
    ParseInt(ParseIntError),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidFormat => write!(f, "Invalid format"),
            ParseError::ParseInt(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for ParseError {}

struct PlainTextDecoder;

impl Decoder for PlainTextDecoder {
    type Item = MyData;
    type Error = ParseError;

    fn decode(&self, data: Bytes) -> Result<Self::Item, Self::Error> {
        let text = String::from_utf8(data.to_vec())
            .map_err(|_| ParseError::InvalidFormat)?;
        let parts: Vec<&str> = text.split(',').collect();
        if parts.len() != 2 {
            return Err(ParseError::InvalidFormat);
        }
        let id = parts[0]
            .strip_prefix("id=")
            .ok_or(ParseError::InvalidFormat)?
            .parse::<u32>()
            .map_err(ParseError::ParseInt)?;
        let name = parts[1]
            .strip_prefix("name=")
            .ok_or(ParseError::InvalidFormat)?
            .to_string();
        Ok(MyData { id, name })
    }
}
```

Then use your custom format with `Pub` and `Sub`:

```rust,no_run
use std::sync::Arc;
use object_transfer::{Pub, Sub, SubOpt};
use object_transfer::traits::PubTrait;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = async_nats::connect("demo.nats.io").await?;
    let js = Arc::new(async_nats::jetstream::new(client));

    // Use your custom plain-text encoder
    let publisher: Pub<MyData, _> = Pub::new(
        js.clone(),
        "events",
        Arc::new(PlainTextEncoder),
    );

    let event = MyData { id: 1, name: "test".to_string() };
    publisher.publish(&event).await?;

    Ok(())
}
```

### Selecting Format at Runtime

Since the encoder and decoder are passed as parameters, you can select the format at runtime:

```rust,no_run
use std::sync::Arc;
use object_transfer::{
  encoders::JSONEncoder,
  encoders::JSONDecoder,
  encoders::MessagePackEncoder,
  encoders::MessagePackDecoder,
  Pub, Sub, SubOpt,
  traits::PubTrait,
};

fn get_encoder(format: &str) -> Arc<dyn object_transfer::encoders::Encoder<Item = MyData> + Send + Sync> {
    match format {
        "json" => Arc::new(JSONEncoder::new()),
        "msgpack" => Arc::new(MessagePackEncoder::new()),
        // Could also include custom formats here
        _ => Arc::new(JSONEncoder::new()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = async_nats::connect("demo.nats.io").await?;
    let js = Arc::new(async_nats::jetstream::new(client));

    let format = std::env::var("MESSAGE_FORMAT").unwrap_or_else(|_| "json".to_string());
    let publisher: Pub<MyData, _> = Pub::new(
        js.clone(),
        "events",
        get_encoder(&format),
    );

    Ok(())
}
```

### Built-in Formats

The library provides built-in encoders/decoders for common formats (when features are enabled):

- **JSON** (feature `json`): [`JSONEncoder`](src/encoders/json.rs) and [`JSONDecoder`](src/encoders/json.rs)
- **MessagePack** (feature `msgpack`): [`MessagePackEncoder`](src/encoders/msgpack.rs) and [`MessagePackDecoder`](src/encoders/msgpack.rs)

use ::std::sync::Arc;
use ::std::time::{SystemTime, UNIX_EPOCH};

use futures::StreamExt;
use serde::{Deserialize, Serialize};

use crate::redis::{Publisher, PublisherConfig, Subscriber, SubscriberConfig};
use crate::traits::SubOptTrait;
use crate::{Format, Pub, PubTrait, Sub, SubTrait};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct MyObj {
  field: String,
}

#[derive(Clone)]
struct TestSubOptions {
  format: Format,
  auto_ack: bool,
}

impl SubOptTrait for TestSubOptions {
  fn get_auto_ack(&self) -> bool {
    self.auto_ack
  }

  fn get_format(&self) -> Format {
    self.format
  }
}

fn unique_stream_name(format: Format) -> String {
  let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_millis();
  format!("object_transfer_redis_{}_{}", format.to_string(), now)
}

async fn setup(format: Format) -> Option<(Pub<MyObj>, Sub<MyObj>)> {
  let client = redis::Client::open("redis://127.0.0.1:6379/").ok()?;
  let con = client.get_multiplexed_async_connection().await.ok()?;
  let stream_name = unique_stream_name(format);
  let publisher_group = format!("{}_publisher", stream_name);
  let subscriber_group = format!("{}_subscriber", stream_name);
  let subscriber_consumer = format!("{}_consumer", stream_name);

  let publisher =
    Publisher::new(&con, PublisherConfig::new().group_name(publisher_group));
  let subscriber = Subscriber::new(
    &con,
    SubscriberConfig::new(stream_name.clone())
      .group_name(subscriber_group)
      .consumer_name(subscriber_consumer)
      .num_fetch(1)
      .block_time(500),
  );

  let options = Arc::new(TestSubOptions {
    format,
    auto_ack: true,
  });
  let pub_typed = Pub::new(Arc::new(publisher), stream_name, format);
  let sub_typed = Sub::new(Arc::new(subscriber), None, options);
  Some((pub_typed, sub_typed))
}

async fn roundtrip(format: Format) {
  if let Some((publisher, reader)) = setup(format).await {
    let obj = MyObj {
      field: "value".into(),
    };
    let sub = ::tokio::spawn(async move {
      let mut subscriber = reader.subscribe().await.unwrap();
      let (obj, _) = subscriber.next().await.unwrap().unwrap();
      obj
    });
    publisher.publish(&obj).await.unwrap();
    let recv = sub.await.unwrap();
    assert_eq!(obj, recv);
  } else {
    panic!("Redis server not available!");
  }
}

#[tokio::test]
async fn test_messagepack() {
  roundtrip(Format::MessagePack).await;
}

#[tokio::test]
async fn test_json() {
  roundtrip(Format::JSON).await;
}

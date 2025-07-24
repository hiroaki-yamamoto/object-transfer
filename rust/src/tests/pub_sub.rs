use ::std::sync::Arc;

use crate::nats::{AckSubOptions, Pub, Sub};
use crate::{Format, PubTrait};
use async_nats::jetstream::{
  consumer::pull::Config as PullConfig, stream::Config as StreamConfig,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct MyObj {
  field: String,
}

async fn setup(format: Format) -> Option<(Pub, Sub<MyObj>)> {
  let client = async_nats::connect_with_options(
    "127.0.0.1:4222",
    async_nats::ConnectOptions::default()
      .retry_on_initial_connect()
      .max_reconnects(5),
  )
  .await
  .unwrap();
  let js = async_nats::jetstream::new(client);
  let name: Arc<str> =
    Arc::from(format!("object_transfer_{}", format.to_string()).as_str());
  let publisher = Pub::new(js.clone(), name.to_string(), format).await.ok()?;
  let subscriber = Sub::new(
    js,
    AckSubOptions::new(format, name.clone())
      .stream_config(StreamConfig {
        name: name.to_string(),
        subjects: vec![name.to_string()],
        ..Default::default()
      })
      .pull_config(PullConfig {
        durable_name: Some(name.to_string()),
        ..Default::default()
      }),
  )
  .await
  .ok()?;
  Some((publisher, subscriber))
}

async fn roundtrip(format: Format) {
  if let Some((publisher, mut subscriber)) = setup(format).await {
    let obj = MyObj {
      field: "value".into(),
    };
    let sub = ::tokio::spawn(async move {
      return subscriber.next().await;
    });
    publisher.publish(&obj).await.unwrap();
    let recv = sub.await.unwrap().unwrap().unwrap();
    assert_eq!(obj, recv);
  } else {
    panic!("NATS server not available!");
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

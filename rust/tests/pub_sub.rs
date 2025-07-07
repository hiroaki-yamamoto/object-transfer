use futures::StreamExt;
use object_transfer::nats::{Pub, Sub};
use object_transfer::{Format, PubTrait};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct MyObj {
  field: String,
}

async fn setup(format: Format) -> Option<(Pub, Sub<MyObj>)> {
  let client = match async_nats::connect("127.0.0.1:4222").await {
    Ok(c) => c,
    Err(_) => return None,
  };
  let js = async_nats::jetstream::new(client);
  let publisher =
    Pub::new(js.clone(), "object_transfer", format).await.ok()?;
  let subscriber = Sub::new(js, format).await.ok()?;
  Some((publisher, subscriber))
}

async fn roundtrip(format: Format) {
  if let Some((publisher, mut subscriber)) = setup(format).await {
    let obj = MyObj {
      field: "value".into(),
    };
    publisher.publish(&obj).await.unwrap();
    let recv = subscriber.next().await.unwrap().unwrap();
    assert_eq!(obj, recv);
  } else {
    eprintln!("NATS server not available; skipping test");
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

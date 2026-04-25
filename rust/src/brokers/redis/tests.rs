use ::std::sync::Arc;
use ::std::time::{SystemTime, UNIX_EPOCH};

use futures::StreamExt;
use serde::{de::Error as DeErr, ser::Error as SeErr};

use crate::options::SubOpt;
use crate::tests::entity::TestEntity;
use crate::{Pub, PubTrait, Sub, SubTrait, UnSubTrait};

use super::{Publisher, PublisherConfig, Subscriber, SubscriberConfig};

use crate::encoder::{
  Decoder as IDecoder, Encoder as IEncoder, JSONDecoder, JSONEncoder,
  MessagePackDecoder, MessagePackEncoder,
};

type EncoderType<E> =
  Arc<dyn IEncoder<Item = TestEntity, Error = E> + Send + Sync>;
type DecoderType<E> =
  Arc<dyn IDecoder<Item = TestEntity, Error = E> + Send + Sync>;

fn unique_stream_name(name: &str) -> String {
  let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_millis();
  format!("object_transfer_redis_{}_{}", name, now)
}

async fn setup<DE, SE>(
  name: &str,
  encoder: EncoderType<SE>,
  decoder: DecoderType<DE>,
) -> Option<(Pub<TestEntity, SE>, Sub<TestEntity, DE>)>
where
  DE: DeErr + Send + Sync,
  SE: SeErr + Send + Sync,
{
  let client = redis::Client::open("redis://127.0.0.1:6379/").ok()?;
  let pub_con = client.get_multiplexed_async_connection().await.ok()?;
  let sub_con = client.get_multiplexed_async_connection().await.ok()?;
  let stream_name = unique_stream_name(name);
  let publisher_group = format!("{}_publisher", stream_name);
  let subscriber_group = format!("{}_subscriber", stream_name);
  let subscriber_consumer = format!("{}_consumer", stream_name);

  let publisher = Publisher::new(
    &pub_con,
    PublisherConfig::new().group_name(publisher_group),
  );
  let subscriber = Arc::new(Subscriber::new(
    &sub_con,
    SubscriberConfig::new(stream_name.clone())
      .group_name(subscriber_group)
      .consumer_name(subscriber_consumer)
      .num_fetch(1)
      .block_time(500),
  ));

  let options = SubOpt::new();
  let pub_typed = Pub::new(Arc::new(publisher), stream_name, encoder);
  let sub_typed =
    Sub::new(subscriber.clone(), subscriber.clone(), decoder, options);
  Some((pub_typed, sub_typed))
}

async fn roundtrip<DE, SE>(
  name: &str,
  encoder: EncoderType<SE>,
  decoder: DecoderType<DE>,
) where
  DE: DeErr + Send + Sync + 'static,
  SE: SeErr + Send + Sync,
{
  if let Some((publisher, reader)) = setup(name, encoder, decoder).await {
    let obj = TestEntity {
      id: 1,
      name: "test".into(),
    };
    let sub = ::tokio::spawn(async move {
      let mut subscriber = reader.subscribe().await.unwrap();
      let (obj, ack) = subscriber.next().await.unwrap().unwrap();
      ack.ack().await.unwrap();
      reader.unsubscribe().await.unwrap();
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
  let encoder = Arc::new(MessagePackEncoder::<TestEntity>::new());
  let decoder = Arc::new(MessagePackDecoder::<TestEntity>::new());
  roundtrip("messagepack", encoder, decoder).await;
}

#[tokio::test]
async fn test_json() {
  let encoder = Arc::new(JSONEncoder::<TestEntity>::new());
  let decoder = Arc::new(JSONDecoder::<TestEntity>::new());
  roundtrip("json", encoder, decoder).await;
}

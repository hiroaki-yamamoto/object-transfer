use ::std::sync::Arc;

use futures::StreamExt;
use serde::{de::Error as DeErr, ser::Error as SeErr};

use crate::encoders::{
  Decoder as IDecoder, Encoder as IEncoder, JSONDecoder, JSONEncoder,
  MessagePackDecoder, MessagePackEncoder,
};
use crate::options::SubOpt;
use crate::tests::entity::TestEntity;
use crate::{Pub, PubTrait, Sub, SubTrait, UnSubTrait};
use async_nats::jetstream::{
  consumer::pull::Config as PullConfig, stream::Config as StreamConfig,
};

use super::super::nats::{SubFetcher, SubFetcherOpt};

async fn setup<SE: SeErr + Send + Sync, DE: DeErr + Send + Sync>(
  name: impl Into<String>,
  encoder: Arc<dyn IEncoder<Item = TestEntity, Error = SE> + Send + Sync>,
  decoder: Arc<dyn IDecoder<Item = TestEntity, Error = DE> + Send + Sync>,
) -> Option<(Pub<TestEntity, SE>, Sub<TestEntity, DE>)> {
  let client = async_nats::connect_with_options(
    "127.0.0.1:4222",
    async_nats::ConnectOptions::default()
      .retry_on_initial_connect()
      .max_reconnects(5),
  )
  .await
  .unwrap();
  let js = Arc::new(async_nats::jetstream::new(client));
  let name: Arc<str> = Arc::from(format!("object_transfer_{}", name.into()));
  let publisher = Pub::new(js.clone(), name.to_string(), encoder);
  let ack_option = SubFetcherOpt::new(name.clone())
    .stream_config(StreamConfig {
      name: name.to_string(),
      subjects: vec![name.to_string()],
      ..Default::default()
    })
    .pull_config(PullConfig {
      durable_name: Some(name.to_string()),
      ..Default::default()
    });
  let sub_option = SubOpt::new();
  let subfetcher = Arc::new(SubFetcher::new(js, ack_option).await.unwrap());
  let reader = Sub::new(subfetcher.clone(), subfetcher, decoder, sub_option);
  Some((publisher, reader))
}

async fn roundtrip<
  SE: SeErr + Send + Sync + 'static,
  DE: DeErr + Send + Sync + 'static,
>(
  name: impl Into<String>,
  encoder: Arc<dyn IEncoder<Item = TestEntity, Error = SE> + Send + Sync>,
  decoder: Arc<dyn IDecoder<Item = TestEntity, Error = DE> + Send + Sync>,
) {
  if let Some((publisher, reader)) = setup(name, encoder, decoder).await {
    let obj = TestEntity {
      id: 42,
      name: "Test Object".to_string(),
    };
    let sub = ::tokio::spawn(async move {
      let mut subscriber = reader.subscribe().await.unwrap();
      let (obj, _) = subscriber.next().await.unwrap().unwrap();
      reader.unsubscribe().await.unwrap();
      obj
    });
    publisher.publish(&obj).await.unwrap();
    let recv = sub.await.unwrap();
    assert_eq!(obj, recv);
  } else {
    panic!("NATS server not available!");
  }
}

#[tokio::test]
async fn test_messagepack() {
  let encoder = Arc::new(MessagePackEncoder::new());
  let decoder = Arc::new(MessagePackDecoder::new());
  roundtrip("messagepack", encoder, decoder).await;
}

#[tokio::test]
async fn test_json() {
  let encoder = Arc::new(JSONEncoder::new());
  let decoder = Arc::new(JSONDecoder::new());
  roundtrip("json", encoder, decoder).await;
}

# タスク

次のようなライブラリを作成してください。

- オブジェクトを入力として、Natsブローカーへ伝達するコード
- Natsブローカーから受信したメッセージをオブジェクトに変換するコード
  - このコードは`futures::prelude::Stream`を使用して、メッセージを受信する。
- サポートさせるシリアライズ/デリアライズ方式は次の通り。
  - MessagePack
  - JSON
- natsブローカーとの接続は、`async`/`await`を使用して非同期で行う。
- SOLID原則に従うこと。
- コードはテスト可能であること
- 外部利用者のためのテストを考慮して、各々の機能に対してtraitを作成すること。
- エラーについては、`thiserror`及び`enum`を使用して、エラーを明確に定義すること。

# ディレクトリ構造
他の言語にも対応できるように、上記ライブラリは`rust`ディレクトリ内で作成する。

## `rust/src`内の構造
|ファイル名|内容|
|---|---|
|`lib.rs`|ライブラリのエントリポイント|
|`traits.rs`|トレイト定義|
|`enum.rs`|列挙型定義|
|`error.rs`|エラー定義|
|`nats.rs`|Natsブローカーとの通信を行うモジュール|
|`nats/sub.rs`|Natsブローカーから受信したメッセージをオブジェクトに変換する|
|`nats/pub.rs`|Natsブローカーへオブジェクトを送信する|

# 例

```rust
use ::object_transfer::nats::Pub;
use ::object_transfer::Format;
use ::object_transfer::PubTrait;
use ::futures::StreamExt;

struct MyObject {
    field: String,
}

async fn test_pub() {
    let clinet = ::async_nats::connect("nats://localhost:4222").await.unwrap();
    let js = async_nats::jetstream::new(clinet);
    let pub_client_msgpack = Pub::new(js, "stream_name", Format::MessagePack).await.unwrap();
    let pub_client_json = Pub::new(js, Format::JSON).await.unwrap();
    let obj = MyObject { field: "value".to_string() };

    // MessagePackでのパブリッシュ
    pub_client_msgpack.publish(&obj).await.unwrap();

    // JSONでのパブリッシュ
    pub_client_json.publish(&obj).await.unwrap();
}

async fn test_sub() {
    let client = ::async_nats::connect("nats://localhost:4222").await.unwrap();
    let js = async_nats::jetstream::new(client, "stream_name");
    let sub_client_msgpack: Sub<MyObject> =
      Sub::new(js, Format::MessagePack).await.unwrap();
    let sub_client_json: Sub<MyObject> =
      Sub::new(js, Format::JSON).await.unwrap();

    // MessagePackでのサブスクライブ
    sub_client_msgpack.next().await.unwrap();

    // JSONでのサブスクライブ
    sub_client_json.next().await.unwrap();
}
```

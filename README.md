# slack_rs

A user-friendly Slack SDK implementation using slack-morphism-rust.

## Implementation Status

- :white_check_mark: 依存関係整理 (slack-morphism-rust, axum)
- :white_check_mark: Socket Mode対応 (feature = "socket_mode")
  - :white_check_mark: メッセージ受信
  - :white_check_mark: コマンド対応
  - :white_check_mark: インタラクション対応
- :white_check_mark: axumでのHTTPサーバー
  - :white_check_mark: Webhookエンドポイント実装
  - :memo: OAuth対応
  - :white_check_mark: イベント受信
- :white_check_mark: ドキュメント整備
  - :white_check_mark: API使用例
  - :white_check_mark: 環境変数設定
  - :white_check_mark: テスト方法

## Features

### Webhook Mode

Webhookを使用してSlackイベントを受信するには、以下の手順で実行します：

1. 環境変数の設定:
```bash
export SLACK_SIGNING_SECRET="your-signing-secret"
```

2. サンプルコードの実行:
```bash
cargo run --example webhook_example
```

### Socket Mode

Socket Modeを使用してSlackイベントを受信するには、以下の手順で実行します：

1. 環境変数の設定:
```bash
export SLACK_APP_TOKEN="xapp-..."
```

2. サンプルコードの実行:
```bash
cargo run --example socket_mode_example --features socket_mode
```

### カスタムイベントハンドラの実装

イベントハンドラを実装することで、Slackイベントの処理をカスタマイズできます。
ハンドラは`SlackEventHandler`トレイトを実装する必要があります。

#### 基本的な使い方

1. ハンドラの実装:
```rust
use slack_rs::{SlackEventHandler, MessageClient};
use async_trait::async_trait;

#[derive(Clone)]
struct CustomHandler;

#[async_trait]
impl SlackEventHandler for CustomHandler {
    async fn handle_event(
        &self,
        event: SlackPushEvent,
        client: &MessageClient,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // イベントの処理をここに実装
        Ok(())
    }
}
```

2. ハンドラの使用:
```rust
// デフォルトパス（/push）を使用する場合
let app = create_app_with_handler(
    SlackSigningSecret::new(signing_secret),
    SlackApiToken::new(bot_token),
    CustomHandler,
);

// カスタムパスを使用する場合
let app = create_app_with_path(
    SlackSigningSecret::new(signing_secret),
    SlackApiToken::new(bot_token),
    CustomHandler,
    "/slack/events",
);
```

#### ハンドラの種類

1. NoopHandler（デフォルト）:
```rust
// イベントを無視するデフォルトのハンドラ
let app = create_app(SlackSigningSecret::new(signing_secret));
```

2. カスタムハンドラ:
```rust
// イベントをカスタム処理するハンドラ
let app = create_app_with_handler(
    SlackSigningSecret::new(signing_secret),
    SlackApiToken::new(bot_token),
    CustomHandler,
);
```

#### イベントの種類と処理パターン

1. URL検証:
```rust
// URL検証は自動的に処理されます
// カスタム処理が必要な場合のみ実装してください
if let SlackPushEvent::UrlVerification(_) = event {
    // URL検証イベントの処理
}
```

2. メンション:
```rust
// メンションイベントの処理例
if let SlackPushEvent::EventCallback(callback) = event {
    if let SlackEventCallbackBody::AppMention(mention) = callback.event {
        // メッセージの送信
        client
            .reply_to_thread(
                mention.channel.as_ref(),
                &mention.origin.ts.to_string(),
                "応答メッセージ",
            )
            .await?;
    }
}
```

#### エラー処理

1. エラーの返却:
```rust
// エラーは`Box<dyn std::error::Error>`として返却します
if something_went_wrong {
    return Err("エラーが発生しました".into());
}
```

2. ログ出力:
```rust
// tracing クレートを使用してログを出力します
tracing::info!("イベントを受信: {:?}", event);
tracing::error!("エラーが発生: {}", error);
```

#### 環境変数

必要な環境変数:
- `SLACK_SIGNING_SECRET`: Slackアプリの署名シークレット
- `SLACK_BOT_TOKEN`: Slackボットのトークン
- `NGROK_AUTHTOKEN`: ngrokのトークン（開発時）
- `NGROK_DOMAIN`: ngrokのドメイン（開発時）

#### メンション応答の例

以下は、メンションされた時に応答するボットの実装例です：

```rust
use slack_rs::{SlackEventHandler, MessageClient};
use async_trait::async_trait;

#[derive(Clone)]
struct MentionHandler;

#[async_trait]
impl SlackEventHandler for MentionHandler {
    async fn handle_event(
        &self,
        event: SlackPushEvent,
        client: &MessageClient,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let SlackPushEvent::EventCallback(callback) = event {
            if let SlackEventCallbackBody::AppMention(mention) = callback.event {
                client
                    .reply_to_thread(
                        mention.channel.as_ref(),
                        &mention.origin.ts.to_string(),
                        "はい、呼びましたか？",
                    )
                    .await?;
            }
        }
        Ok(())
    }
}

// ハンドラの使用
let app = create_app_with_handler(
    SlackSigningSecret::new(signing_secret),
    SlackApiToken::new(bot_token),
    MentionHandler,
);
```

### メンション応答の例

メンションされた時のみ応答するボットを実装する例です。以下の手順で実行します：

#### 必要な環境変数
- `SLACK_SIGNING_SECRET`: Slackアプリの署名シークレット
- `SLACK_BOT_TOKEN`: Slackボットのトークン
- `NGROK_AUTHTOKEN`: ngrokの認証トークン
- `NGROK_DOMAIN`: ngrokの固定ドメイン（例：arriving-informally-manatee.ngrok-free.app）

#### Slackアプリの設定
1. Event Subscriptionsを有効化
2. Request URLを設定: `https://{NGROK_DOMAIN}/push`
   - URLが有効であることを確認（チャレンジレスポンスが成功すること）
3. 以下のBot Event Scopesを追加:
   - `app_mentions:read`: メンションの検知用
   - `chat:write`: メッセージ送信用

**注意**: 権限を更新した場合は、必ずワークスペースからBotを一度削除し、再インストールしてください。
権限の更新は再インストール後に反映されます。

#### 使用方法
1. `.env`ファイルに環境変数を設定:
```bash
SLACK_SIGNING_SECRET=your-signing-secret
SLACK_BOT_TOKEN=xoxb-your-bot-token
NGROK_AUTHTOKEN=your-ngrok-token
NGROK_DOMAIN=your-ngrok-domain
```

2. チャンネルの準備:
   - ボットをチャンネルに招待: `/invite @[BOT名]`
   - チャンネルIDの確認方法は「検証手順」セクションを参照

3. サーバーを起動:
```bash
cargo run --example mention_response --features events
```

4. 動作確認:
   - Slackでボットをメンション（@）する
   - ボットが「はい、呼びましたか？」と応答することを確認
   - 応答はスレッド内で行われます

Slack APIの使いやすいSDKを提供するRustライブラリです。[slack-morphism-rust](https://github.com/abdolence/slack-morphism-rust)をベースに、特にSocket Modeを使用したメッセージの受信に焦点を当てています。

## 実装タスク

### 基本設定
- :memo: slack-morphism-rustライブラリの依存関係追加
- :memo: tokio, hyper等の基本的な依存関係の設定
- :memo: 環境変数設定の仕組み作成（SLACK_TEST_APP_TOKEN等）

### メッセージ送信機能
- :white_check_mark: メッセージ送信の基本機能実装
  - :white_check_mark: テキストメッセージの送信
  - :white_check_mark: リッチメッセージ（ブロックキット）の送信
  - :white_check_mark: ファイル添付機能
  - :white_check_mark: スレッド返信機能
  - :white_check_mark: メッセージの更新・削除機能
- :white_check_mark: メッセージ送信のエラーハンドリング
- :white_check_mark: レートリミット対応
- :white_check_mark: メッセージ送信のユーティリティ関数の提供

#### 使用例
```rust
use slack_rs::{MessageClient, SlackApiToken, SlackApiTokenValue};
use slack_morphism::blocks::{SlackBlock, SlackBlockText, SlackSectionBlock};

// クライアントの初期化
let token = std::env::var("SLACK_BOT_TOKEN").expect("SLACK_BOT_TOKEN must be set");
let token = SlackApiToken::new(SlackApiTokenValue(token));
let client = MessageClient::new(token);

// テキストメッセージの送信
let message = client.send_text("C1234567890", "基本的なテキストメッセージ").await?;

// ブロックキットを使用したメッセージ
let blocks = vec![
    SlackBlock::Section(
        SlackSectionBlock::new().with_text(SlackBlockText::MarkDown("*太字* _斜体_ ~取り消し線~".into()))
    ),
    SlackBlock::Section(
        SlackSectionBlock::new().with_text(SlackBlockText::Plain("プレーンテキスト".into()))
    ),
];
client.send_blocks("C1234567890", blocks).await?;

// スレッド返信
client.reply_to_thread("C1234567890", &message.ts.to_string(), "スレッドへの返信").await?;

// メッセージの更新と削除
let message = client.send_text("C1234567890", "このメッセージは更新されます").await?;
client.update_message("C1234567890", &message.ts.to_string(), "更新されたメッセージ").await?;
client.delete_message("C1234567890", &message.ts.to_string()).await?;

// ファイルのアップロード
let file_content = "テストファイルの内容".as_bytes().to_vec();
client.upload_file(vec!["C1234567890".to_string()], file_content, "test.txt").await?;
```

### 検証手順

1. Botの設定
   - 以下の権限が必要です：
     - `channels:read`: チャンネル情報の読み取り
     - `chat:write`: メッセージの送信
     - `files:write`: ファイルのアップロード
   - **重要**: 権限を更新した場合は、必ずワークスペースからBotを一度削除し、再インストールしてください。
     権限の更新は再インストール後に反映されます。

2. チャンネルの準備
   - チャンネルIDの確認方法：
     1. Slackでチャンネルを開く
     2. チャンネル名をクリック
     3. 「チャンネルの詳細」の下部にチャンネルID（例：C1234567890）が表示されます
   - Botをチャンネルに招待：
     - `/invite @[BOT名]` コマンドを使用
     または
     - チャンネルの設定から「メンバーを追加する」でBotを追加

3. 環境変数の設定
```bash
export SLACK_BOT_TOKEN=xoxb-your-token
```

4. サンプルコードの実行
```bash
cargo run --example message_sending
```

**注意**: チャンネルIDは実際のSlackワークスペースのチャンネルIDに置き換えてください。

### Socket Mode実装
- :memo: Socket Modeクライアントの基本構造体の定義
- :memo: Socket Modeを使用したメッセージ受信ロジックの実装
- :memo: メッセージハンドラーの実装
- :memo: エラーハンドリングの実装
- :memo: 非同期処理の最適化

### テストとサンプル実装
- :memo: Socket Mode実装のユニットテスト作成
- :memo: Socket Modeを使用したメッセージ受信のサンプルコード作成
- :memo: サンプルコードの実行テスト作成
- :memo: テストケースのドキュメント作成


## 使用方法

### 環境設定

1. Slack APIトークンの取得
   - Slack Appを作成し、Socket Modeを有効化
   - App-Level Tokenを発行（`xapp-`で始まるトークン）

2. 環境変数の設定
```bash
export SLACK_TEST_APP_TOKEN=xapp-your-token-here
```

### 基本的な使用例

#### Webhookモード

```rust
use slack_rs::{create_app, create_app_with_path};
use axum::{routing::get, Router};
use slack_morphism::prelude::*;

#[tokio::main]
async fn main() {
    // 環境変数からSlack署名シークレットを取得
    let signing_secret = std::env::var("SLACK_SIGNING_SECRET")
        .expect("SLACK_SIGNING_SECRETが設定されていません");

    // デフォルトパス（/push）を使用する場合
    let router = Router::new()
        .route("/health", get(|| async { "OK" }))
        .merge(create_app(SlackSigningSecret::new(signing_secret.clone())));

    // カスタムパスを使用する場合（例：/slack/events）
    let router = Router::new()
        .route("/health", get(|| async { "OK" }))
        .merge(create_app_with_path(SlackSigningSecret::new(signing_secret), "/slack/events"));

    // サーバーの起動
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}
```

#### Socket Modeを使用したメッセージ受信の例
// ※実装完了後に追加予定
```

## コンポーネント構成

### Webhookクライアント

Webhookクライアントは、Slack Events APIからのイベントを受信・処理するための主要コンポーネントです。
以下の機能を提供します：

- イベント受信エンドポイント（`/push`）
- Slack署名検証
- イベントタイプに基づく処理の振り分け
- URL検証チャレンジへの対応

### Socket Modeクライアント

Socket Modeクライアントは、Slackのリアルタイムメッセージを受信するための主要コンポーネントです。
実装完了後、こちらに具体的な使用方法と設定例を追加します。

### エラーハンドリング

エラーハンドリングの方針と実装例については、実装完了後に追加します。

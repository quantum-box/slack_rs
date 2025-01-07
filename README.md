# Slack RS

Rustで実装されたSlack APIクライアントライブラリです。

## 実装タスク

📝 **実装タスク一覧**

✅ 基本構造の実装
- ✅ Slack Event構造体の定義
- ✅ Webhookハンドラーの実装
- ✅ テストケースの作成（署名検証、メッセージイベント）

✅ 主要機能の実装
- ✅ Slack Webhookエンドポイントの実装
- ✅ イベント署名検証の実装
- ✅ エラーハンドリングの追加

📝 ユースケースの実装
- ✅ メッセージイベントの処理
- 📝 その他のイベントタイプの対応
- ✅ 単体テストの作成

## 主要な要素

- **Webhook Handler**: Slack Events APIからのイベントを受信・処理するエンドポイント
- **署名検証**: Slackからのリクエストの真正性を検証する機能
- **イベント処理**: 受信したイベントの種類に応じた処理の実装

## 環境変数

- `SLACK_SIGNING_SECRET`: Slackアプリケーションの署名シークレット（必須）

## `SlackEvent` 構造体

`SlackEvent`構造体は、Slackから受信したイベントデータを表現します。

```rust
pub struct SlackEvent {
    #[serde(rename = "type")]
    event_type: String,
    event: Option<SlackMessageEvent>,
}

pub struct SlackMessageEvent {
    #[serde(rename = "type")]
    event_type: String,
    channel: String,
    text: Option<String>,
    user: Option<String>,
    ts: String,
}
```

- `event_type`: イベントの種類（例: "event_callback"）
- `event`: メッセージイベントの詳細情報

### Webhookエンドポイント

- `/slack/events`: Slack Events APIからのイベントを受信するエンドポイント
  - POSTリクエストを受け付け
  - 署名検証を実施
  - イベントの種類に応じた処理を実行

## PlantUML ダイアグラム

```plantuml
@startuml
// コンポーネントの関係を表すダイアグラム
@enduml
```

この図は、[ダイアグラムが表現している内容の説明]を示しています。

## ユースケース

[コンポーネント名]を使用した具体的なユースケースを以下に示します。

### 1. [ユースケース1の名前]

**目的**: [ユースケースの目的を説明]

**実装タスク**:
- 📝 [具体的な実装タスク1]
- 📝 [具体的な実装タスク2]
- 📝 テストケースの作成

**ユースケース例**:
- [具体例1] 
- [具体例2]
- [具体例3]

```plantuml
@startuml
// ユースケースの関係を表すダイアグラム
@enduml
```

このシステムでは、[ユースケースの詳細な説明]:

- `[コンポーネント1]`: [役割の説明]
  - `[ツール1]`: [機能の説明]
  - `[ツール2]`: [機能の説明]

### 2. [ユースケース2の名前]

[以下同様]

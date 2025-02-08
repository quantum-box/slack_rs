#[cfg(feature = "events")]
use crate::events::Event;
use crate::{
    message::MessageClient,
    types::{SigningSecret, Token},
};
use async_trait::async_trait;
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use bytes::Bytes;
use slack_morphism::{
    events::SlackPushEvent as MorphismPushEvent, signature_verifier::SlackEventSignatureVerifier,
};
use std::time::{SystemTime, UNIX_EPOCH};

// SlackApiSignatureVerifier is already available through prelude

/// 何もしないデフォルトのイベントハンドラ
#[derive(Clone)]
pub struct NoopHandler;

#[cfg(feature = "events")]
#[async_trait]
impl SlackEventHandler for NoopHandler {
    async fn handle_event(
        &self,
        _event: Event,
        _client: &MessageClient,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Slackイベントのハンドラトレイト
///
/// このトレイトを実装することで、Slackイベントの処理をカスタマイズできます。
/// イベントハンドラは`Send + Sync + Clone + 'static`を実装する必要があります。
#[async_trait]
pub trait SlackEventHandler: Send + Sync + Clone + 'static {
    /// イベントを処理します
    ///
    /// # 引数
    /// * `event` - 処理対象のSlackイベント
    /// * `client` - メッセージ送信用のクライアント
    ///
    /// # 戻り値
    /// * `Ok(())` - イベントの処理に成功
    /// * `Err(Box<dyn std::error::Error>)` - イベントの処理に失敗
    #[cfg(feature = "events")]
    async fn handle_event(
        &self,
        event: Event,
        client: &MessageClient,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

/// デフォルトのwebhookエンドポイントパス
pub const DEFAULT_WEBHOOK_PATH: &str = "/push";

#[derive(Clone)]
#[cfg(feature = "events")]
pub struct AppState<H: SlackEventHandler> {
    pub signing_secret: SigningSecret,
    pub message_client: MessageClient,
    pub handler: H,
}

/// Slackからのwebhookイベントを処理します。
///
/// このハンドラは以下の機能を提供します：
/// - リクエストの署名検証
/// - URL検証チャレンジへの応答
/// - イベントのJSONパース
///
/// イベントの具体的な処理は、ライブラリ利用者が実装する必要があります。
///
/// # URL検証
/// Slackからの検証リクエストに対して、チャレンジ値をプレーンテキストで返します。
/// Content-Typeは`text/plain`である必要があります。
///
/// # エラー処理
/// - チャレンジ値が空の場合は400 Bad Requestを返します
/// - 署名が無効な場合は401 Unauthorizedを返します
#[cfg(feature = "events")]
pub async fn handle_push_event<H: SlackEventHandler>(
    State(state): State<AppState<H>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // 生のリクエストボディを取得
    let body_str = String::from_utf8(body.to_vec()).unwrap_or_default();
    tracing::debug!("受信したボディ: {}", body_str);

    // 署名の検証
    let signature = headers
        .get(SlackEventSignatureVerifier::SLACK_SIGNED_HASH_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let timestamp = headers
        .get(SlackEventSignatureVerifier::SLACK_SIGNED_TIMESTAMP)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // タイムスタンプの検証（5分以上古いリクエストは拒否）
    let timestamp_num = timestamp.parse::<u64>().unwrap_or(0);
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if current_time.abs_diff(timestamp_num) > 300 {
        tracing::error!("リクエストが古すぎます: timestamp={}", timestamp);
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from("Request timestamp is too old"))
            .unwrap();
    }

    tracing::debug!(
        "署名を検証: signature={}, timestamp={}",
        signature,
        timestamp
    );

    // 署名の検証
    let verifier = SlackEventSignatureVerifier::new(&state.signing_secret.into());
    let verification_result = verifier.verify(signature, &body_str, timestamp).is_ok();

    if !verification_result {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from("Invalid signature"))
            .unwrap();
    }

    // ボディをSlackPushEventとしてパース
    let morphism_event: MorphismPushEvent = match serde_json::from_str(&body_str) {
        Ok(event) => event,
        Err(e) => {
            tracing::error!("JSONのパースに失敗: {}", e);
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Invalid JSON"))
                .unwrap();
        }
    };

    tracing::info!("Slackイベントを受信: type={:?}", morphism_event);
    tracing::debug!("イベントの詳細: {:?}", morphism_event);

    let event: Event = morphism_event.into();
    match &event {
        Event::UrlVerification { challenge } => {
            tracing::info!("URL検証イベントを受信: challenge={}", challenge);
            if challenge.is_empty() {
                tracing::error!("チャレンジ値が空です");
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from("Challenge value is missing"))
                    .unwrap();
            }
            // プレーンテキストでチャレンジ値を返す
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .body(Body::from(challenge.clone()))
                .unwrap()
        }
        Event::AppMention { .. } | Event::Message { .. } => {
            tracing::info!("イベントコールバックを受信");
            if let Err(e) = state
                .handler
                .handle_event(event, &state.message_client)
                .await
            {
                tracing::error!("イベントの処理に失敗: {}", e);
            }
            Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap()
        }
        Event::Other => {
            tracing::debug!("未対応のイベントタイプを受信");
            Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap()
        }
    }
}

/// webhookエンドポイントを作成します。
///
/// # Arguments
/// * `signing_secret` - Slack署名シークレット
#[cfg(feature = "events")]
pub fn create_app(signing_secret: SigningSecret) -> Router {
    create_app_with_handler(signing_secret, Token::new(""), NoopHandler)
}

/// webhookエンドポイントをカスタムハンドラで作成します。
///
/// # Arguments
/// * `signing_secret` - Slack署名シークレット
/// * `bot_token` - Slackボットトークン
/// * `handler` - イベントを処理するハンドラ
#[cfg(feature = "events")]
pub fn create_app_with_handler<H: SlackEventHandler>(
    signing_secret: SigningSecret,
    bot_token: Token,
    handler: H,
) -> Router {
    create_app_with_path(signing_secret, bot_token, handler, DEFAULT_WEBHOOK_PATH)
}

/// webhookエンドポイントを指定したパスで作成します。
///
/// # Arguments
/// * `signing_secret` - Slack署名シークレット
/// * `bot_token` - Slackボットトークン
/// * `handler` - イベントを処理するハンドラ
/// * `path` - webhookエンドポイントのパス（例："/push" や "/slack/events"）
#[cfg(feature = "events")]
pub fn create_app_with_path<H: SlackEventHandler>(
    signing_secret: SigningSecret,
    bot_token: Token,
    handler: H,
    path: &str,
) -> Router {
    let state = AppState {
        signing_secret,
        message_client: MessageClient::new(bot_token),
        handler,
    };
    Router::new()
        .route(
            path,
            post(|state, headers, body| handle_push_event::<H>(state, headers, body)),
        )
        .with_state(state)
}

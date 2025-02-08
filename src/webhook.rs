use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use bytes::Bytes;
use slack_morphism::{prelude::*, signature_verifier::SlackEventSignatureVerifier};
use std::time::{SystemTime, UNIX_EPOCH};

// SlackApiSignatureVerifier is already available through prelude

/// デフォルトのwebhookエンドポイントパス
pub const DEFAULT_WEBHOOK_PATH: &str = "/push";

#[derive(Clone)]
pub struct AppState {
    pub signing_secret: SlackSigningSecret,
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
pub async fn handle_push_event(
    State(state): State<AppState>,
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
    let verifier = SlackEventSignatureVerifier::new(&state.signing_secret);
    let verification_result = verifier.verify(signature, &body_str, timestamp).is_ok();

    if !verification_result {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from("Invalid signature"))
            .unwrap();
    }

    // ボディをSlackPushEventとしてパース
    let event: SlackPushEvent = match serde_json::from_str(&body_str) {
        Ok(event) => event,
        Err(e) => {
            tracing::error!("JSONのパースに失敗: {}", e);
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Invalid JSON"))
                .unwrap();
        }
    };

    tracing::info!("Slackイベントを受信: type={:?}", event);
    tracing::debug!("イベントの詳細: {:?}", event);

    match event {
        SlackPushEvent::UrlVerification(ref url_ver) => {
            tracing::info!("URL検証イベントを受信: challenge={}", url_ver.challenge);
            tracing::debug!("URL検証の詳細: event={:?}", event);
            if url_ver.challenge.is_empty() {
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
                .body(Body::from(url_ver.challenge.clone()))
                .unwrap()
        }
        SlackPushEvent::EventCallback(callback) => {
            tracing::info!("イベントコールバックを受信: {:?}", callback);
            Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap()
        }
        _ => {
            tracing::debug!("未対応のイベントタイプを受信");
            Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap()
        }
    }
}

/// メッセージ送信機能なしのwebhookエンドポイントを作成します。
pub fn create_webhook_app(signing_secret: SlackSigningSecret) -> Router {
    create_webhook_app_with_path(signing_secret, DEFAULT_WEBHOOK_PATH)
}

/// webhookエンドポイントを作成します。
pub fn create_app(signing_secret: SlackSigningSecret) -> Router {
    create_app_with_path(signing_secret, DEFAULT_WEBHOOK_PATH)
}

/// webhookエンドポイントを指定したパスで作成します。
///
/// # Arguments
/// * `signing_secret` - Slack署名シークレット
/// * `path` - webhookエンドポイントのパス（例："/push" や "/slack/events"）
pub fn create_app_with_path(signing_secret: SlackSigningSecret, path: &str) -> Router {
    let state = AppState { signing_secret };
    Router::new()
        .route(path, post(handle_push_event))
        .with_state(state)
}

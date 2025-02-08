use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use slack_morphism::prelude::*;
use slack_morphism::signature_verifier::SlackEventSignatureVerifier;
// SlackApiSignatureVerifier is already available through prelude

/// デフォルトのwebhookエンドポイントパス
pub const DEFAULT_WEBHOOK_PATH: &str = "/push";

#[derive(Clone)]
pub struct AppState {
    pub signing_secret: SlackSigningSecret,
}

/// Slackからのwebhookイベントを処理します。
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
    Json(event): Json<SlackPushEvent>,
) -> impl IntoResponse {
    tracing::info!("Slackイベントを受信: type={:?}", event);
    tracing::debug!("イベントの詳細: {:?}", event);

    // 署名の検証
    let signature = headers
        .get(SlackEventSignatureVerifier::SLACK_SIGNED_HASH_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let timestamp = headers
        .get(SlackEventSignatureVerifier::SLACK_SIGNED_TIMESTAMP)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    tracing::debug!(
        "署名を検証: signature={}, timestamp={}",
        signature,
        timestamp
    );

    let body_str = serde_json::to_string(&event).unwrap_or_default();

    // 署名の検証
    let verifier = SlackEventSignatureVerifier::new(&state.signing_secret);
    let verification_result = verifier.verify(signature, &body_str, timestamp).is_ok();

    if !verification_result {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from("Invalid signature"))
            .unwrap();
    }

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
            // チャレンジ値をそのまま返す（Slackの要件）
            url_ver.challenge.into_response()
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

pub fn create_app(signing_secret: SlackSigningSecret) -> Router {
    create_app_with_path(signing_secret, DEFAULT_WEBHOOK_PATH)
}

/// 指定したパスでwebhookエンドポイントを作成します。
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

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

pub async fn handle_push_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(event): Json<SlackPushEvent>,
) -> impl IntoResponse {
    tracing::info!("Received Slack event: {:?}", event);

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
        "Verifying signature: {}, timestamp: {}",
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
        SlackPushEvent::UrlVerification(url_ver) => {
            tracing::info!("URL検証イベントを受信: {}", url_ver.challenge);
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/plain")
                .body(Body::from(url_ver.challenge))
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

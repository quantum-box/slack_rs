use axum::{
    Router,
    routing::post,
    Json,
    response::IntoResponse,
    http::{StatusCode, HeaderMap, header},
    extract::State,
};
use serde::{Deserialize, Serialize};
use slack_morphism::prelude::*;
// SlackApiSignatureVerifier is already available through prelude

#[derive(Clone)]
pub struct AppState {
    pub signing_secret: SlackSigningSecret,
}

pub async fn handle_push_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(event): Json<SlackPushEvent>,
) -> impl IntoResponse {
    // 署名の検証
    let signature = headers
        .get("X-Slack-Signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let timestamp = headers
        .get("X-Slack-Request-Timestamp")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let body_str = serde_json::to_string(&event).unwrap_or_default();
    
    let verifier = SlackApiSigningVerifier::new(&state.signing_secret);
    if !verifier.verify(signature, timestamp, body_str.as_bytes()) {
        return (StatusCode::UNAUTHORIZED, "Invalid signature").into_response();
    }

    match event {
        SlackPushEvent::UrlVerification(url_ver) => {
            println!("URL検証イベントを受信: {}", url_ver.challenge);
            (StatusCode::OK, [(header::CONTENT_TYPE, "text/plain")], url_ver.challenge).into_response()
        }
        SlackPushEvent::CallbackEvent(callback) => {
            println!("イベントコールバックを受信: {:?}", callback);
            Json(serde_json::json!({})).into_response()
        }
        _ => (StatusCode::OK, Json(serde_json::json!({}))).into_response(),
    }
}

pub fn create_app(signing_secret: SlackSigningSecret) -> Router {
    let state = AppState { signing_secret };
    Router::new()
        .route("/push", post(handle_push_event))
        .with_state(state)
}

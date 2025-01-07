use axum::{
    Router,
    routing::post,
    Json,
    response::{IntoResponse, Response},
    http::{StatusCode, HeaderMap, header},
    extract::State,
};
use serde::{Deserialize, Serialize};
use slack_morphism::prelude::*;
use slack_morphism::crypto::hmac_sha256;

#[derive(Debug, Deserialize, Serialize)]
pub struct UrlVerification {
    pub token: String,
    pub challenge: String,
    #[serde(rename = "type")]
    pub event_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EventCallback {
    pub token: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub event: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SlackEvent {
    UrlVerification(UrlVerification),
    EventCallback(EventCallback),
}

#[derive(Clone)]
pub struct AppState {
    pub signing_secret: SlackSigningSecret,
}

pub async fn handle_push_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(event): Json<SlackEvent>,
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
    
    let verification = SlackRequestVerification::new(
        signature,
        timestamp,
        &body_str,
        &state.signing_secret,
    );
    
    if verification.is_err() {
        return (StatusCode::UNAUTHORIZED, "Invalid signature").into_response();
    }

    match event {
        SlackEvent::UrlVerification(url_ver) => {
            println!("URL検証イベントを受信: {}", url_ver.challenge);
            (StatusCode::OK, [(header::CONTENT_TYPE, "text/plain")], url_ver.challenge).into_response()
        }
        SlackEvent::EventCallback(callback) => {
            println!("イベントコールバックを受信: {:?}", callback);
            Json(serde_json::json!({})).into_response()
        }
    }
}

pub fn create_app(signing_secret: SlackSigningSecret) -> Router {
    let state = AppState { signing_secret };
    Router::new()
        .route("/push", post(handle_push_event))
        .with_state(state)
}

use axum::{
    Router,
    routing::post,
    Json,
    response::{IntoResponse, Response},
    http::{StatusCode, HeaderMap, header},
    extract::State,
};
use serde::Deserialize;
use slack_morphism::prelude::*;

#[derive(Debug, Deserialize)]
pub struct UrlVerification {
    pub token: String,
    pub challenge: String,
    #[serde(rename = "type")]
    pub event_type: String,
}

#[derive(Debug, Deserialize)]
pub struct EventCallback {
    pub token: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub event: serde_json::Value,
}

#[derive(Debug, Deserialize)]
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
    
    if let Err(_) = state.signing_secret.verify_signature(signature, timestamp, &body_str) {
        return (StatusCode::UNAUTHORIZED, "Invalid signature").into_response();
    }

    match event {
        SlackEvent::UrlVerification(url_ver) => {
            println!("URL検証イベントを受信: {}", url_ver.challenge);
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/plain")
                .body(url_ver.challenge)
                .unwrap()
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

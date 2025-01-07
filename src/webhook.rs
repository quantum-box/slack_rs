use axum::{
    Router,
    routing::post,
    Json,
    response::IntoResponse,
    http::{StatusCode, HeaderMap},
    extract::State,
};
use serde::Deserialize;
use slack_morphism::prelude::*;
use std::sync::Arc;

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
    State(_state): State<AppState>,
    _headers: HeaderMap,
    Json(event): Json<SlackEvent>,
) -> impl IntoResponse {
    // TODO: 署名の検証を実装

    match event {
        SlackEvent::UrlVerification(url_ver) => {
            println!("URL検証イベントを受信: {}", url_ver.challenge);
            (StatusCode::OK, Json(serde_json::json!({ "challenge": url_ver.challenge })))
        }
        SlackEvent::EventCallback(callback) => {
            println!("イベントコールバックを受信: {:?}", callback);
            (StatusCode::OK, Json(serde_json::json!({})))
        }
    }
}

pub fn create_app(signing_secret: SlackSigningSecret) -> Router {
    let state = AppState { signing_secret };
    Router::new()
        .route("/push", post(handle_push_event))
        .with_state(state)
}

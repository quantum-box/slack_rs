use axum::{
    extract::{State, RawBody},
    http::StatusCode,
    routing::post,
    Router,
};
use serde::Deserialize;
use slack_morphism::signature_verifier::*;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct SlackEvent {
    #[serde(rename = "type")]
    event_type: String,
    event: Option<SlackMessageEvent>,
}

#[derive(Debug, Deserialize)]
struct SlackMessageEvent {
    #[serde(rename = "type")]
    event_type: String,
    channel: String,
    text: Option<String>,
    user: Option<String>,
    ts: String,
}

#[derive(Clone)]
pub struct AppState {
    pub signing_secret: String,
}

pub async fn slack_events_handler(
    State(app_state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    RawBody(body): RawBody,
) -> StatusCode {
    let timestamp = headers.get("X-Slack-Request-Timestamp");
    let signature = headers.get("X-Slack-Signature");

    // Slack署名検証
    if let (Some(ts), Some(sig)) = (timestamp, signature) {
        let ts_str = ts.to_str().unwrap_or("");
        let sig_str = sig.to_str().unwrap_or("");
        let verifier = SlackEventSignatureVerifier::new(&app_state.signing_secret);
        let body_bytes = body.bytes().await.unwrap_or_default();
        let body_str = String::from_utf8_lossy(&body_bytes);
        
        match verifier.verify(ts_str, body_str.as_ref(), sig_str) {
            Ok(_) => {
                // イベントをデシリアライズ
                match serde_json::from_slice::<SlackEvent>(&body_bytes) {
                    Ok(event) => {
                        if let Some(message_event) = event.event {
                            if message_event.event_type == "message" {
                                tracing::info!(
                                    "Received message event: channel={}, user={:?}, text={:?}",
                                    message_event.channel,
                                    message_event.user,
                                    message_event.text
                                );
                                return StatusCode::OK;
                            }
                        }
                        tracing::debug!("Received non-message event: {:?}", event);
                        StatusCode::OK
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse Slack event: {}", e);
                        StatusCode::BAD_REQUEST
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Invalid Slack signature: {}", e);
                StatusCode::UNAUTHORIZED
            }
        }
    } else {
        tracing::warn!("Missing required Slack headers");
        StatusCode::BAD_REQUEST
    }
}

pub fn slack_router() -> Router<Arc<AppState>> {
    Router::new().route("/slack/events", post(slack_events_handler))
}

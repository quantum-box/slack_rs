use axum::{
    body::Bytes,
    extract::{State},
    http::StatusCode,
    routing::post,
    Router,
};
use hmac::Mac;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing;

#[derive(Debug, Deserialize, Serialize)]
pub struct SlackEvent {
    #[serde(rename = "type")]
    event_type: String,
    event: Option<SlackMessageEvent>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SlackMessageEvent {
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

#[axum::debug_handler]
pub async fn slack_events_handler(
    State(app_state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> StatusCode {
    let timestamp = headers.get("X-Slack-Request-Timestamp");
    let signature = headers.get("X-Slack-Signature");

    // Slack署名検証
    if let (Some(ts), Some(sig)) = (timestamp, signature) {
        let ts_str = ts.to_str().unwrap_or("");
        let sig_str = sig.to_str().unwrap_or("");
        // Verify signature using HMAC-SHA256
        let base_string = format!("v0:{}:{}", ts_str, String::from_utf8_lossy(&body));
        let mut mac = <hmac::Hmac<sha2::Sha256> as Mac>::new_from_slice(app_state.signing_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(base_string.as_bytes());
        let result = mac.finalize();
        let expected_sig = format!("v0={}", hex::encode(result.into_bytes()));
        
        if sig_str == expected_sig {
            // Parse JSON after signature verification
            match serde_json::from_slice::<SlackEvent>(&body) {
                Ok(event) => {
                    if let Some(ref message_event) = event.event {
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
                    tracing::warn!("Failed to parse event JSON: {}", e);
                    StatusCode::UNPROCESSABLE_ENTITY
                }
            }
        } else {
            tracing::warn!("Invalid Slack signature");
            StatusCode::UNAUTHORIZED
        }
    } else {
        tracing::warn!("Missing required Slack headers");
        StatusCode::BAD_REQUEST
    }
}

pub fn slack_router() -> Router<Arc<AppState>> {
    Router::new().route("/slack/events", post(slack_events_handler))
}

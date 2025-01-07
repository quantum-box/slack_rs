use crate::webhook::*;
use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode},
};
use hmac::Mac;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_invalid_signature() {
    let state = Arc::new(AppState {
        signing_secret: "test_secret".to_string(),
    });

    let mut headers = HeaderMap::new();
    headers.insert("X-Slack-Request-Timestamp", "12345".parse().unwrap());
    headers.insert("X-Slack-Signature", "v0=invalid".parse().unwrap());

    let app = slack_router().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/slack/events")
                .header("X-Slack-Request-Timestamp", "12345")
                .header("X-Slack-Signature", "v0=invalid")
                .header("Content-Type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_valid_message_event() {
    let state = Arc::new(AppState {
        signing_secret: "test_secret".to_string(),
    });

    let timestamp = "12345";
    let body = json!({
        "type": "event_callback",
        "event": {
            "type": "message",
            "channel": "C1234567890",
            "user": "U1234567890",
            "text": "Hello, world!",
            "ts": "1234567890.123456"
        }
    });
    let body_str = serde_json::to_string(&body).unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("X-Slack-Request-Timestamp", timestamp.parse().unwrap());

    // Create valid signature using HMAC-SHA256
    let base_string = format!("v0:{}:{}", timestamp, body_str);
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice("test_secret".as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(base_string.as_bytes());
    let result = mac.finalize();
    let sig = format!("v0={}", hex::encode(result.into_bytes()));
    headers.insert("X-Slack-Signature", sig.parse().unwrap());

    let app = slack_router().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/slack/events")
                .header("X-Slack-Request-Timestamp", timestamp)
                .header("X-Slack-Signature", &sig)
                .header("Content-Type", "application/json")
                .body(Body::from(body_str))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

use axum::{
    Router,
    routing::post,
    Json,
    response::{IntoResponse, Response},
    body::Body,
    http::{StatusCode, HeaderMap, header},
    extract::State,
};
use slack_morphism::{
    prelude::*,
    api::SlackRequestVerificationHeader,
};
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
    
    // 署名の検証
    let verification_header = SlackRequestVerificationHeader::new(signature, timestamp);
    if !verification_header.verify(&state.signing_secret, body_str.as_bytes()) {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from("Invalid signature"))
            .unwrap();
    }

    match event {
        SlackPushEvent::UrlVerification(url_ver) => {
            println!("URL検証イベントを受信: {}", url_ver.challenge);
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/plain")
                .body(Body::from(url_ver.challenge))
                .unwrap()
        }
        SlackPushEvent::CallbackEvent(callback) => {
            println!("イベントコールバックを受信: {:?}", callback);
            Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap()
        }
        _ => Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .unwrap(),
    }
}

pub fn create_app(signing_secret: SlackSigningSecret) -> Router {
    let state = AppState { signing_secret };
    Router::new()
        .route("/push", post(handle_push_event))
        .with_state(state)
}

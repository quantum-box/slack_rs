use axum::{
    Router,
    routing::post,
    Json,
    response::IntoResponse,
    http::{StatusCode, HeaderMap, header, Response},
    extract::State,
};
use hyper::Body;
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
    
    // 署名の検証
    if !state.signing_secret.verify_request(signature, timestamp, body_str.as_bytes()) {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from("Invalid signature"))
            .unwrap()
            .into_response();
    }

    match event {
        SlackPushEvent::UrlVerification(url_ver) => {
            println!("URL検証イベントを受信: {}", url_ver.challenge);
            Response::new(Body::from(url_ver.challenge))
        }
        SlackPushEvent::CallbackEvent(callback) => {
            println!("イベントコールバックを受信: {:?}", callback);
            Response::new(Body::empty())
        }
        _ => Response::new(Body::empty()),
    }
}

pub fn create_app(signing_secret: SlackSigningSecret) -> Router {
    let state = AppState { signing_secret };
    Router::new()
        .route("/push", post(handle_push_event))
        .with_state(state)
}

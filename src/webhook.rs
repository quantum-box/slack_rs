use axum::{
    Router,
    routing::post,
    Extension,
    response::IntoResponse,
    http::{StatusCode, Response},
};
use slack_morphism::prelude::*;
use std::sync::Arc;

type SlackEnv = SlackClientEventsListenerEnvironment<SlackClientHyperConnector>;

#[axum::debug_handler]

pub async fn handle_push_event(
    Extension(_env): Extension<Arc<SlackEnv>>,
    Extension(event): Extension<SlackPushEvent>,
) -> impl IntoResponse {
    println!("Received push event: {:?}", event);
    match event {
        SlackPushEvent::UrlVerification(url_ver) => {
            // SlackのURL検証用
            Response::builder()
                .status(StatusCode::OK)
                .body(url_ver.challenge)
                .unwrap()
        }
        SlackPushEvent::EventCallback(callback) => {
            // メッセージイベントなどをここで確認
            println!("Callback event: {:?}", callback);
            // TODO: 必要に応じてwebhook先に通知する処理などを追加
            Response::builder()
                .status(StatusCode::OK)
                .body(String::new())
                .unwrap()
        }
        _ => Response::builder()
            .status(StatusCode::OK)
            .body(String::new())
            .unwrap(),
    }
}

pub fn create_app(signing_secret: SlackSigningSecret) -> Router {
    let client = SlackClient::new(
        SlackClientHyperConnector::new()
            .expect("Failed to create HTTP connector")
    );
    let listener = SlackEventsAxumListener::new(
        Arc::new(
            SlackClientEventsListenerEnvironment::new(Arc::new(client))
        )
    );

    Router::new()
        .route(
            "/push",
            post(handle_push_event).layer(
                listener
                    .events_layer(&signing_secret)
                    .with_event_extractor(SlackEventsExtractors::push_event())
            )
        )
}

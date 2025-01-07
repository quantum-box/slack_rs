use axum::{Router, routing::post, Extension};
use slack_morphism::prelude::*;
use hyper::{Body, Response};
use std::sync::Arc;

pub async fn handle_push_event(
    Extension(_env): Extension<Arc<SlackClientEventsListenerEnvironment<SlackClientHyperConnector>>>,
    Extension(event): Extension<SlackPushEvent>,
) -> Response<Body> {
    println!("Received push event: {:?}", event);
    match event {
        SlackPushEvent::UrlVerification(url_ver) => {
            // SlackのURL検証用
            Response::new(Body::from(url_ver.challenge))
        }
        SlackPushEvent::CallbackEvent(callback) => {
            // メッセージイベントなどをここで確認
            println!("Callback event: {:?}", callback);
            // TODO: 必要に応じてwebhook先に通知する処理などを追加
            Response::new(Body::empty())
        }
        _ => Response::new(Body::empty()),
    }
}

pub fn create_app(signing_secret: SlackSigningSecret) -> Router {
    let listener = SlackEventsAxumListener::new(
        Arc::new(
            SlackClientEventsListenerEnvironment::new(
                Arc::new(SlackClient::new(SlackClientHyperConnector::new().unwrap()))
            )
        )
    );

    Router::new()
        .route(
            "/push",
            post(handle_push_event)
                .layer(
                    listener
                        .events_layer(&signing_secret)
                        .with_event_extractor(SlackEventsExtractors::push_event())
                )
        )
}

use axum::{
    Router,
    routing::post,
    Extension,
    response::IntoResponse,
    http::{StatusCode, Response},
};
use slack_morphism::prelude::*;
use std::sync::Arc;

pub async fn handle_push_event(
    Extension(_env): Extension<Arc<SlackClientEventsListenerEnvironment<SlackClientHyperConnector<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>>>>,
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
    let connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_only()
        .enable_http1()
        .build();

    let listener = SlackEventsAxumListener::new(
        Arc::new(
            SlackClientEventsListenerEnvironment::new(
                Arc::new(SlackClient::new(SlackClientHyperConnector::new(connector)))
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

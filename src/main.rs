#[cfg(feature = "events")]
mod events;
#[cfg(feature = "oauth")]
mod oauth;
mod webhook;

use axum::{routing::get, Router};
use slack_morphism::prelude::*;
use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::webhook::create_app as create_webhook_app;

#[tokio::main]
async fn main() {
    // Initialize logging
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .compact()
        .init();

    let mut app = Router::new().route("/health", get(|| async { "OK" }));

    // Webhookエンドポイントの設定
    let signing_secret = SlackSigningSecret::new(
        std::env::var("SLACK_SIGNING_SECRET")
            .expect("SLACK_SIGNING_SECRETが設定されていません")
            .into(),
    );
    app = app.merge(create_webhook_app(signing_secret));

    // Configure server address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("サーバーを開始します: {}", addr);

    // Start the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

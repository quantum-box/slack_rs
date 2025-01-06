mod oauth;

use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::oauth::{oauth_router, OAuthConfig};

#[tokio::main]
async fn main() {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .compact()
        .init();

    // OAuth設定の初期化
    let oauth_config = OAuthConfig::new(
        std::env::var("SLACK_CLIENT_ID")
            .expect("SLACK_CLIENT_IDが設定されていません"),
        std::env::var("SLACK_CLIENT_SECRET")
            .expect("SLACK_CLIENT_SECRETが設定されていません"),
        "http://localhost:3000/oauth/callback".to_string(),
    );

    // Initialize the router with OAuth routes
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .merge(oauth_router(oauth_config));

    // Configure server address
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("サーバーを開始します: {}", addr);

    // Start the server
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

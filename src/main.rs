use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .compact()
        .init();

    // Initialize the router with a basic health check endpoint
    let app = Router::new()
        .route("/health", get(|| async { "OK" }));

    // Configure server address
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("サーバーを開始します: {}", addr);

    // Start the server
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

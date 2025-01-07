use slack_rs::webhook::{slack_router, AppState};
use std::{env, net::SocketAddr, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let signing_secret =
        env::var("SLACK_SIGNING_SECRET").expect("SLACK_SIGNING_SECRET must be set");

    let state = Arc::new(AppState { signing_secret });

    let app = slack_router().with_state(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    tracing::info!("Starting server on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

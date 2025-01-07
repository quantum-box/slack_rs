use axum::{routing::get, Router};
use slack_morphism::prelude::*;
use slack_rs::create_app;
use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    // ロギングの初期化
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .compact()
        .init();

    info!("Webhookサーバーを起動します");

    // 環境変数からSlack署名シークレットを取得
    let signing_secret =
        std::env::var("SLACK_SIGNING_SECRET").expect("SLACK_SIGNING_SECRETが設定されていません");

    // ルーターの設定
    let router = Router::new()
        .route("/health", get(|| async { "OK" }))
        .merge(create_app(SlackSigningSecret::new(signing_secret)));

    // サーバーアドレスの設定
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("サーバーを開始します: {}", addr);

    // サーバーの起動
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

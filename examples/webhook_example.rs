use axum::{routing::get, Router};
use ngrok::prelude::*;
use slack_rs::{create_app, SigningSecret};
use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
#[ignore = "外部サービス（ngrok）を使用するため、CIでは実行しない"]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // ロギングの初期化
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .compact()
        .init();

    info!("Webhookサーバーを起動します");

    // 環境変数からSlack署名シークレットを取得
    let signing_secret =
        std::env::var("SLACK_SIGNING_SECRET").expect("SLACK_SIGNING_SECRETが設定されていません");

    let ngrok_domain = std::env::var("NGROK_DOMAIN").expect("NGROK_DOMAINが設定されていません");

    // ルーターの設定
    let router = Router::new()
        .route("/health", get(|| async { "OK" }))
        .merge(create_app(SigningSecret::new(signing_secret)));

    // サーバーアドレスの設定
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("サーバーを開始します: {}", addr);

    let mut tun = ngrok::Session::builder()
        .authtoken_from_env()
        .connect()
        .await?
        .http_endpoint()
        .domain(ngrok_domain)
        .listen()
        .await?;

    info!("Tunnel URL: {}", tun.url());
    tun.forward_http(addr).await?;

    // サーバーの起動
    axum_server::bind(addr)
        .serve(router.into_make_service())
        .await?;
    Ok(())
}

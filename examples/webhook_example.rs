use axum::{routing::get, Router};
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use ngrok::prelude::*;
use slack_rs::{create_app, SigningSecret};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::Service;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
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

    let tun = ngrok::Session::builder()
        // Read the token from the NGROK_AUTHTOKEN environment variable
        .authtoken_from_env()
        // Connect the ngrok session
        .connect()
        .await?
        // Start a tunnel with an HTTP edge
        .http_endpoint()
        .domain(ngrok_domain)
        .listen()
        .await?;

    info!("Tunnel URL: {}", tun.url());

    // サーバーの起動
    loop {
        let (socket, _) = tun.accept().await?;
        let io = TokioIo::new(socket);
        let router = router.clone();
        
        tokio::task::spawn(async move {
            let service = service_fn(move |req| {
                let mut router = router.clone();
                async move { Service::call(&mut router, req).await }
            });
            
            if let Err(err) = Builder::new(TokioExecutor::new())
                .serve_connection(io, service)
                .await
            {
                eprintln!("Error serving connection: {}", err);
            }
        });
    }

    #[allow(unreachable_code)]
    Ok(())
}

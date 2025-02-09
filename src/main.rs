use axum::{routing::get, Router};
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use slack_rs::{create_app, SigningSecret};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::Service as TowerService;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    // このファイルは基本的な使用例を示すためのものです。
    // より詳細な使用例については examples/ ディレクトリを参照してください。

    // ロギングの初期化
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .compact()
        .init();

    info!("基本的なWebhookサーバーを起動します");

    // 環境変数からSlack署名シークレットを取得
    let signing_secret =
        std::env::var("SLACK_SIGNING_SECRET").expect("SLACK_SIGNING_SECRETが設定されていません");

    // ルーターの設定
    let router = Router::new()
        .route("/health", get(|| async { "OK" }))
        .merge(create_app(SigningSecret::new(signing_secret)));

    // サーバーアドレスの設定
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("サーバーを開始します: {}", addr);

    // サーバーの起動
    let listener = TcpListener::bind(addr).await.unwrap();
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(socket);
        let service = router.clone();

        tokio::task::spawn(async move {
            let service = service_fn(move |req| {
                let mut service = service.clone();
                async move { TowerService::call(&mut service, req).await }
            });

            if let Err(err) = Builder::new(TokioExecutor::new())
                .serve_connection(io, service)
                .await
            {
                eprintln!("Error serving connection: {}", err);
            }
        });
    }
}

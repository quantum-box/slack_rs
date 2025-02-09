use axum::{routing::get, Router};
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::Service;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Hello axum" }));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server started, ready to accept connections");

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(socket);
        let app = app.clone();

        tokio::task::spawn(async move {
            let service = service_fn(move |req| {
                let mut app = app.clone();
                async move { Service::call(&mut app, req).await }
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

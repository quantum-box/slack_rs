use std::env;
use tracing::info;

#[cfg(feature = "socket_mode")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Set up environment variables if not already set
    if env::var("SLACK_APP_TOKEN").is_err() {
        info!("Please set SLACK_APP_TOKEN environment variable");
        std::process::exit(1);
    }

    info!("Starting Socket Mode example...");

    // Run the socket mode listener
    slack_rs::socket_mode::run_socket_mode().await?;

    Ok(())
}

#[cfg(not(feature = "socket_mode"))]
fn main() {
    println!("This example requires the 'socket_mode' feature");
    std::process::exit(1);
}

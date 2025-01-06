use dotenvy::dotenv;
use slack_rs::socket_mode::SocketModeClient;
use std::env;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Get the Slack App Token from environment variables
    let app_token = env::var("SLACK_APP_TOKEN").expect("SLACK_APP_TOKEN must be set in .env file");

    println!("Starting Socket Mode client...");

    // Create and start the Socket Mode client
    let client = SocketModeClient::new(&app_token);
    client.connect().await?;

    // Keep the connection alive
    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");

    Ok(())
}

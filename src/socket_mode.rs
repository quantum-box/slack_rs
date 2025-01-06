//! Socket Mode implementation for Slack WebSocket connections.
//! This module is only available when the "socket_mode" feature is enabled.

use slack_morphism::prelude::*;
use std::error::Error;

/// A client for Slack's Socket Mode connections.
#[cfg(feature = "socket_mode")]
pub struct SocketModeClient {
    app_token: String,
}

#[cfg(feature = "socket_mode")]
impl SocketModeClient {
    /// Creates a new Socket Mode client with the given app token.
    pub fn new(app_token: &str) -> Self {
        Self {
            app_token: app_token.to_string(),
        }
    }

    /// Connects to Slack's Socket Mode WebSocket server.
    pub async fn connect(&self) -> Result<(), Box<dyn Error>> {
        let client = SlackClient::new(SlackClientHyperConnector::new());
        let session = client.open_socket_mode(&self.app_token).await?;
        
        println!("Connected to Slack Socket Mode");
        
        // Keep the connection alive until dropped
        session.run_loop().await?;
        
        Ok(())
    }
}

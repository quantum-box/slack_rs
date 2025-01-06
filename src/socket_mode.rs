//! Socket Mode implementation for Slack WebSocket connections.
//! This module is only available when the "socket_mode" feature is enabled.

use slack_morphism::listener::SlackClientEventsListenerEnvironment;
use slack_morphism::prelude::*;
use slack_morphism::socket_mode::{
    SlackClientSocketModeConfig, SlackClientSocketModeListener, SlackSocketModeListenerCallbacks,
};
use std::error::Error;
use std::sync::Arc;

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
        let http_client = SlackClientHyperConnector::new()?;
        let client = Arc::new(SlackClient::new(http_client));

        let token_value = SlackApiTokenValue(self.app_token.clone());
        let token = SlackApiToken::new(token_value);

        let config = SlackClientSocketModeConfig::new();
        let callbacks = SlackSocketModeListenerCallbacks::new();

        let env = Arc::new(SlackClientEventsListenerEnvironment::new(client.clone()));

        let listener = SlackClientSocketModeListener::new(&config, env, callbacks);

        println!("Connected to Slack Socket Mode");
        listener.listen_for(&token).await?;

        Ok(())
    }
}

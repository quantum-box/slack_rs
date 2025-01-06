//! Socket Mode implementation for Slack WebSocket connections.
//! This module is only available when the "socket_mode" feature is enabled.

use slack_morphism::listener::SlackClientEventsListenerEnvironment;
use slack_morphism::prelude::*;
use slack_morphism::socket_mode::{
    SlackClientSocketModeConfig, SlackClientSocketModeListener, SlackSocketModeListenerCallbacks,
};
use std::error::Error;
use std::sync::Arc;
use tracing::info;
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;

const TEST_CHANNEL: &str = "C06MYKV9YS4"; // Replace with your test channel ID

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
    pub async fn connect(&self) -> Result<Arc<SlackClient<SlackClientHyperConnector<HttpsConnector<HttpConnector>>>>, Box<dyn Error>> {
        let http_client = SlackClientHyperConnector::new()?;
        let client = Arc::new(SlackClient::new(http_client));

        let token_value = SlackApiTokenValue(self.app_token.clone());
        let token = SlackApiToken::new(token_value);

        let config = SlackClientSocketModeConfig::new();
        let callbacks = SlackSocketModeListenerCallbacks::new()
            .with_hello_events(|event, _ctx, _state| async move {
                info!("Socket Mode connection established: {:?}", event);
            })
            .with_push_events(|envelope, _ctx, _state| async move {
                info!("Received event: {:?}", envelope);
                Ok(())
            })
            .with_command_events(|event, _ctx, _state| async move {
                info!("Received command event: {:?}", event);
                Ok(SlackCommandEventResponse::new(
                    SlackMessageContent::new().with_text("Command received".into()),
                ))
            })
            .with_interaction_events(|event, _ctx, _state| async move {
                info!("Received interaction event: {:?}", event);
                Ok(())
            });

        let env = Arc::new(SlackClientEventsListenerEnvironment::new(client.clone()));
        let listener = SlackClientSocketModeListener::new(&config, env, callbacks);

        info!("Connected to Slack Socket Mode");
        tokio::spawn(async move {
            if let Err(e) = listener.listen_for(&token).await {
                info!("Socket mode listener error: {:?}", e);
            }
        });

        Ok(client)
    }

    /// Sends a test message to verify the connection is working
    pub async fn send_test_message(&self, client: Arc<SlackClient<SlackClientHyperConnector<HttpsConnector<HttpConnector>>>>) -> Result<(), Box<dyn Error>> {
        let token = SlackApiToken::new(SlackApiTokenValue(self.app_token.clone()));
        
        let content = SlackMessageContent::new()
            .with_text("Test message from Socket Mode client".into());
        
        let request = SlackApiChatPostMessageRequest::new(
            SlackChannelId(TEST_CHANNEL.to_string()),
            content,
        );

        let _response = client.chat_post_message(&request, &token).await?;
        info!("Test message sent successfully");
        
        Ok(())
    }
}

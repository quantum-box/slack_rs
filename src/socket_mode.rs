//! Socket Mode implementation for Slack WebSocket connections.
//! This module is only available when the "socket_mode" feature is enabled.

use slack_morphism::{
    prelude::*,
    socket_mode::{SlackSocketModeListenerCallbacks, SlackSocketModeListener, SlackSocketModeListenerSettings},
    hyper_tokio::SlackClientHyperConnector,
};
use std::error::Error;
use std::sync::Arc;
use tracing::info;

/// Type alias for a Slack client using Hyper
type SlackHyperClient = SlackClient<SlackClientHyperConnector>;

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
    pub async fn connect(&self) -> Result<Arc<SlackHyperClient>, Box<dyn Error>> {
        let client = Arc::new(SlackClient::new(SlackClientHyperConnector::new()?));
        let token = SlackApiToken::new(SlackApiTokenValue(self.app_token.clone()));

        let callbacks = SlackSocketModeListenerCallbacks::new()
            .with_hello_events(|event| async move {
                info!("Socket Mode connection established: {:?}", event);
                Ok(())
            })
            .with_push_events(|envelope| async move {
                info!("Received event: {:?}", envelope);
                Ok(())
            })
            .with_command_events(|event| async move {
                info!("Received command event: {:?}", event);
                Ok(())
            })
            .with_interaction_events(|event| async move {
                info!("Received interaction event: {:?}", event);
                Ok(())
            });

        let listener = SlackSocketModeListener::new(
            &client,
            &token,
            SlackSocketModeListenerSettings::new(),
            callbacks,
        );

        tokio::spawn(async move {
            if let Err(e) = listener.listen().await {
                info!("Socket mode listener error: {:?}", e);
            }
        });

        info!("Connected to Slack Socket Mode");
        Ok(client)
    }

    /// Sends a test message to verify the connection is working
    pub async fn send_test_message(&self, client: Arc<SlackHyperClient>) -> Result<(), Box<dyn Error>> {
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

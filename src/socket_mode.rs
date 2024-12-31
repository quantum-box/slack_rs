#[cfg(feature = "socket_mode")]
use slack_morphism::{
    events::SlackEventCallbackBody,
    prelude::*,
    socket_mode::{
        SlackClientSocketModeConfig, SlackClientSocketModeListener,
        SlackSocketModeListenerCallbacks,
    },
};
#[cfg(feature = "socket_mode")]
use std::sync::Arc;
#[cfg(feature = "socket_mode")]
use tracing::info;

#[cfg(feature = "socket_mode")]
type Error = Box<dyn std::error::Error + Send + Sync>;
#[cfg(feature = "socket_mode")]
type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "socket_mode")]
pub async fn run_socket_mode() -> Result<()> {
    // Create SlackClient instance
    let client = Arc::new(SlackClient::new(SlackClientHyperConnector::new()?));

    // Get App Level Token from environment
    let app_token_value: SlackApiTokenValue = std::env::var("SLACK_APP_TOKEN")?.into();
    let app_token = SlackApiToken::new(app_token_value);

    // Create socket mode callbacks
    let socket_mode_callbacks = SlackSocketModeListenerCallbacks::new()
        .with_hello_events(|event, _ctx, _state| async move {
            info!("Socket Mode connection established: {:?}", event);
        })
        .with_push_events(|envelope, _ctx, _state| async move {
            info!("Received event: {:?}", envelope);
            process_event(envelope).await
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

    // Create socket mode listener
    let listener_environment = Arc::new(SlackClientEventsListenerEnvironment::new(client.clone()));

    let socket_mode_config = SlackClientSocketModeConfig::new();
    let socket_mode_listener = SlackClientSocketModeListener::new(
        &socket_mode_config,
        listener_environment.clone(),
        socket_mode_callbacks,
    );

    // Start listening
    socket_mode_listener.listen_for(&app_token).await?;

    Ok(())
}

#[cfg(feature = "socket_mode")]
async fn process_event(event: SlackPushEventCallback) -> Result<()> {
    // Implement event processing logic here
    match &event.event {
        SlackEventCallbackBody::Message(msg) => {
            info!("Received message event: {:?}", msg);
        }
        SlackEventCallbackBody::ReactionAdded(reaction) => {
            info!("Received reaction event: {:?}", reaction);
        }
        other => {
            info!("Received other event: {:?}", other);
        }
    }
    Ok(())
}

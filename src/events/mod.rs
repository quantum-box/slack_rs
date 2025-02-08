#[cfg(feature = "events")]
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
#[cfg(feature = "events")]
use serde::Serialize;
#[cfg(feature = "events")]
use slack_morphism::{
    api::SlackApiChatPostMessageRequest,
    events::{SlackEventCallbackBody, SlackPushEvent},
    SlackApiToken, SlackApiTokenValue, SlackClient, SlackMessageContent,
};
#[cfg(feature = "events")]
use tracing::{error, info};

#[cfg(feature = "events")]
use crate::oauth::OAuthConfig;

/// Slackから受信するイベントの種類を表す列挙型
#[derive(Debug, Clone)]
#[cfg(feature = "events")]
pub enum Event {
    /// URL検証イベント
    UrlVerification {
        /// チャレンジ値
        challenge: String,
    },
    /// メンションイベント
    AppMention {
        /// チャンネルID
        channel: String,
        /// メッセージのタイムスタンプ
        ts: String,
        /// メッセージのテキスト
        text: String,
        /// チームID
        team_id: Option<String>,
    },
    /// メッセージイベント
    Message {
        /// チャンネルID
        channel: String,
        /// メッセージのテキスト
        text: String,
        /// チームID
        team_id: Option<String>,
    },
    /// その他のイベント
    Other,
}

#[cfg(feature = "events")]
impl From<SlackPushEvent> for Event {
    fn from(event: SlackPushEvent) -> Self {
        match event {
            SlackPushEvent::UrlVerification(ver) => Self::UrlVerification {
                challenge: ver.challenge,
            },
            SlackPushEvent::EventCallback(callback) => match callback.event {
                SlackEventCallbackBody::AppMention(mention) => Self::AppMention {
                    channel: mention.channel.to_string(),
                    ts: mention.origin.ts.to_string(),
                    text: mention.content.text.expect("メンションテキストが空です"),
                    team_id: Some(callback.team_id.to_string()),
                },
                SlackEventCallbackBody::Message(message) => Self::Message {
                    channel: message
                        .origin
                        .channel
                        .expect("チャンネルIDが空です")
                        .to_string(),
                    text: message.content.unwrap().text.unwrap_or_default(),
                    team_id: Some(callback.team_id.to_string()),
                },
                _ => Self::Other,
            },
            _ => Self::Other,
        }
    }
}

#[cfg(feature = "events")]
#[derive(Debug, Serialize)]
struct ChallengeResponse {
    challenge: String,
}

#[cfg(feature = "events")]
pub fn events_router(config: OAuthConfig) -> Router {
    Router::new()
        .route("/slack/events", post(handle_slack_event))
        .with_state(config)
}

#[cfg(feature = "events")]
async fn handle_slack_event(
    State(config): State<OAuthConfig>,
    Json(event): Json<SlackPushEvent>,
) -> Result<Response, String> {
    info!("Slackイベントを受信: {:?}", event);

    let event: Event = event.into();
    match event {
        Event::UrlVerification { challenge } => {
            Ok(Json(ChallengeResponse { challenge }).into_response())
        }
        Event::Message {
            channel,
            text,
            team_id,
        } => {
            if let Err(e) = handle_message_event(channel, text, team_id, config).await {
                error!("メッセージイベントの処理に失敗: {}", e);
            }
            Ok("ok".into_response())
        }
        Event::AppMention {
            channel,
            text,
            team_id,
            ..
        } => {
            if let Err(e) = handle_app_mention_event(channel, text, team_id, config).await {
                error!("アプリメンションイベントの処理に失敗: {}", e);
            }
            Ok("ok".into_response())
        }
        Event::Other => {
            info!("未対応のイベントタイプ");
            Ok("ok".into_response())
        }
    }
}

#[cfg(feature = "events")]
async fn handle_message_event(
    channel: String,
    _text: String,
    team_id: Option<String>,
    config: OAuthConfig,
) -> Result<(), String> {
    if let Some(team_id) = team_id {
        let connector = SlackClientHyperConnector::new()
            .map_err(|e| format!("Failed to create connector: {}", e))?;
        let client = SlackClient::new(connector);

        // トークンの取得
        let storage = config.token_storage.read().await;
        if let Some(token_response) = storage.get_token(&team_id) {
            let token = SlackApiToken::new(SlackApiTokenValue(token_response.access_token.clone()));
            let session = client.open_session(&token);

            // メッセージの送信
            let post_request = SlackApiChatPostMessageRequest::new(
                channel.into(),
                SlackMessageContent::new().with_text("メッセージを受信しました！".to_string()),
            );

            session
                .chat_post_message(&post_request)
                .await
                .map_err(|e| {
                    error!("メッセージの送信に失敗: {}", e);
                    format!("メッセージの送信に失敗: {}", e)
                })?;
        }
    }
    Ok(())
}

#[cfg(feature = "events")]
async fn handle_app_mention_event(
    channel: String,
    _text: String,
    team_id: Option<String>,
    config: OAuthConfig,
) -> Result<(), String> {
    if let Some(team_id) = team_id {
        let connector = SlackClientHyperConnector::new()
            .map_err(|e| format!("Failed to create connector: {}", e))?;
        let client = SlackClient::new(connector);

        // トークンの取得
        let storage = config.token_storage.read().await;
        if let Some(token_response) = storage.get_token(&team_id) {
            let token = SlackApiToken::new(SlackApiTokenValue(token_response.access_token.clone()));
            let session = client.open_session(&token);

            // メンションへの応答
            let post_request = SlackApiChatPostMessageRequest::new(
                channel.into(),
                SlackMessageContent::new().with_text("はい、呼びましたか？".to_string()),
            );

            session
                .chat_post_message(&post_request)
                .await
                .map_err(|e| {
                    error!("メッセージの送信に失敗: {}", e);
                    format!("メッセージの送信に失敗: {}", e)
                })?;
        }
    }
    Ok(())
}

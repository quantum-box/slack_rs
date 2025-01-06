#[cfg(feature = "events")]
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use slack_morphism::{api::SlackApiChatPostMessageRequest, prelude::*};
use tracing::{error, info};

use crate::oauth::OAuthConfig;

#[cfg(feature = "events")]
#[derive(Debug, Deserialize)]
struct SlackEventWrapper {
    #[serde(rename = "type")]
    event_type: String,
    challenge: Option<String>,
    event: Option<SlackEvent>,
    team_id: Option<String>,
}

#[cfg(feature = "events")]
#[derive(Debug, Deserialize)]
struct SlackEvent {
    #[serde(rename = "type")]
    event_type: String,
    text: Option<String>,
    channel: Option<String>,
    user: Option<String>,
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
    Json(payload): Json<SlackEventWrapper>,
) -> Result<Response, String> {
    info!("Slackイベントを受信: {:?}", payload);

    // URL Verification チャレンジの処理
    if payload.event_type == "url_verification" {
        if let Some(challenge) = payload.challenge {
            return Ok(Json(ChallengeResponse { challenge }).into_response());
        }
    }

    // イベントの処理
    if let Some(event) = payload.event {
        match event.event_type.as_str() {
            "message" => {
                if let Err(e) = handle_message_event(event, payload.team_id, config).await {
                    error!("メッセージイベントの処理に失敗: {}", e);
                }
            }
            "app_mention" => {
                if let Err(e) = handle_app_mention_event(event, payload.team_id, config).await {
                    error!("アプリメンションイベントの処理に失敗: {}", e);
                }
            }
            _ => {
                info!("未対応のイベントタイプ: {}", event.event_type);
            }
        }
    }

    Ok(()).map_err(|e: Box<dyn std::error::Error>| format!("イベント処理エラー: {}", e))?;
    Ok("ok".into_response())
}

#[cfg(feature = "events")]
async fn handle_message_event(
    event: SlackEvent,
    team_id: Option<String>,
    config: OAuthConfig,
) -> Result<(), String> {
    if let (Some(channel), Some(team_id)) = (event.channel, team_id) {
        let connector = SlackClientHyperConnector::new()
            .map_err(|e| format!("Failed to create connector: {}", e))?;
        let client = SlackClient::new(connector);

        // トークンの取得
        let storage = config.token_storage.read().await;
        if let Some(token_response) = storage.get_token(&team_id) {
            let token = SlackApiToken::new(token_response.access_token.clone());
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
            Ok(())
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

#[cfg(feature = "events")]
async fn handle_app_mention_event(
    event: SlackEvent,
    team_id: Option<String>,
    config: OAuthConfig,
) -> Result<(), String> {
    if let (Some(channel), Some(team_id)) = (event.channel, team_id) {
        let connector = SlackClientHyperConnector::new()
            .map_err(|e| format!("Failed to create connector: {}", e))?;
        let client = SlackClient::new(connector);

        // トークンの取得
        let storage = config.token_storage.read().await;
        if let Some(token_response) = storage.get_token(&team_id) {
            let token = SlackApiToken::new(token_response.access_token.clone());
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
            Ok(())
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

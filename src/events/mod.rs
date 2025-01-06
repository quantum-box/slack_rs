use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use slack_morphism::{
    api::SlackApiChatPostMessageRequest,
    prelude::*,
};
use tracing::{error, info};

use crate::oauth::OAuthConfig;

#[derive(Debug, Deserialize)]
struct SlackEventWrapper {
    #[serde(rename = "type")]
    event_type: String,
    challenge: Option<String>,
    event: Option<SlackEvent>,
    team_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SlackEvent {
    #[serde(rename = "type")]
    event_type: String,
    text: Option<String>,
    channel: Option<String>,
    user: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChallengeResponse {
    challenge: String,
}

pub fn events_router(config: OAuthConfig) -> Router {
    Router::new()
        .route("/slack/events", post(handle_slack_event))
        .with_state(config)
}

async fn handle_slack_event(
    State(config): State<OAuthConfig>,
    Json(payload): Json<SlackEventWrapper>,
) -> Response {
    info!("Slackイベントを受信: {:?}", payload);

    // URL Verification チャレンジの処理
    if payload.event_type == "url_verification" {
        if let Some(challenge) = payload.challenge {
            return Json(ChallengeResponse { challenge }).into_response();
        }
    }

    // イベントの処理
    if let Some(event) = payload.event {
        match event.event_type.as_str() {
            "message" => {
                handle_message_event(event, payload.team_id, config).await;
            }
            "app_mention" => {
                handle_app_mention_event(event, payload.team_id, config).await;
            }
            _ => {
                info!("未対応のイベントタイプ: {}", event.event_type);
            }
        }
    }

    "ok".into_response()
}

async fn handle_message_event(event: SlackEvent, team_id: Option<String>, config: OAuthConfig) {
    if let (Some(channel), Some(team_id)) = (event.channel, team_id) {
        let client = SlackClient::new(SlackClientHyperConnector::new());
        
        // トークンの取得
        let storage = config.token_storage.read().await;
        if let Some(token_response) = storage.get_token(&team_id) {
            let session = client.open_session(&token_response.access_token);
            
            // メッセージの送信
            let post_request = SlackApiChatPostMessageRequest::new(
                channel.into(),
                "メッセージを受信しました！".into(),
            );
            
            if let Err(e) = session.chat_post_message(&post_request).await {
                error!("メッセージの送信に失敗: {}", e);
            }
        }
    }
}

async fn handle_app_mention_event(event: SlackEvent, team_id: Option<String>, config: OAuthConfig) {
    if let (Some(channel), Some(team_id)) = (event.channel, team_id) {
        let client = SlackClient::new(SlackClientHyperConnector::new());
        
        // トークンの取得
        let storage = config.token_storage.read().await;
        if let Some(token_response) = storage.get_token(&team_id) {
            let session = client.open_session(&token_response.access_token);
            
            // メンションへの応答
            let post_request = SlackApiChatPostMessageRequest::new(
                channel.into(),
                "はい、呼びましたか？".into(),
            );
            
            if let Err(e) = session.chat_post_message(&post_request).await {
                error!("メッセージの送信に失敗: {}", e);
            }
        }
    }
}

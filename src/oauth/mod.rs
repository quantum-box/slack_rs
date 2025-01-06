#[cfg(feature = "oauth")]
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use slack_morphism::{
    api::SlackOAuthV2AccessTokenRequest, api::SlackOAuthV2AccessTokenResponse, prelude::*,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{error, info};

#[cfg(feature = "oauth")]
// トークンストレージ
#[derive(Default)]
pub struct TokenStorage {
    tokens: HashMap<String, SlackOAuthV2AccessTokenResponse>,
}

#[cfg(feature = "oauth")]
impl TokenStorage {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
        }
    }

    pub fn store_token(&mut self, team_id: String, token: SlackOAuthV2AccessTokenResponse) {
        self.tokens.insert(team_id, token);
    }

    pub fn get_token(&self, team_id: &str) -> Option<&SlackOAuthV2AccessTokenResponse> {
        self.tokens.get(team_id)
    }
}

#[cfg(feature = "oauth")]
// OAuth設定
#[derive(Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub token_storage: Arc<RwLock<TokenStorage>>,
}

#[cfg(feature = "oauth")]
impl OAuthConfig {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            token_storage: Arc::new(RwLock::new(TokenStorage::new())),
        }
    }
}

#[cfg(feature = "oauth")]
// OAuthルーターの設定
pub fn oauth_router(config: OAuthConfig) -> Router {
    Router::new()
        .route("/oauth/start", get(oauth_start))
        .route("/oauth/callback", get(oauth_callback))
        .with_state(config)
}

#[cfg(feature = "oauth")]
// OAuth開始ハンドラー
async fn oauth_start(State(config): State<OAuthConfig>) -> Response {
    let scopes = vec!["channels:history", "chat:write", "commands"];

    let auth_url = format!(
        "https://slack.com/oauth/v2/authorize?client_id={}&scope={}",
        config.client_id,
        scopes.join(",")
    );

    info!("OAuth開始: {}", auth_url);
    axum::response::Redirect::temporary(&auth_url).into_response()
}

#[cfg(feature = "oauth")]
// OAuthコールバックハンドラー
async fn oauth_callback(
    State(config): State<OAuthConfig>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, String> {
    if let Some(code) = params.get("code") {
        info!("OAuth認証コードを受信: {}", code);

        let connector = SlackClientHyperConnector::new()
            .map_err(|e| format!("Failed to create connector: {}", e))?;
        let client = SlackClient::new(connector);

        // アクセストークンの取得
        let result = client
            .oauth2_access(&SlackOAuthV2AccessTokenRequest::new(
                config.client_id.as_str().into(),
                config.client_secret.as_str().into(),
                code.into(),
            ))
            .await;

        match result {
            Ok(token_response) => {
                let team_info = token_response.team.clone();
                let team_id = team_info.id.to_string();
                let mut storage = config.token_storage.write().await;
                storage.store_token(team_id.clone(), token_response);
                info!("チーム {} のトークンを保存しました", team_id);
                Ok("OAuth認証成功！".into_response())
            }
            Err(e) => {
                error!("OAuth認証エラー: {}", e);
                Ok(format!("OAuth認証エラー: {}", e).into_response())
            }
        }
    } else {
        Ok("認証コードが見つかりません".into_response())
    }
}

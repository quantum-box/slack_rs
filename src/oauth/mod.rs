use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use slack_morphism::{
    prelude::*,
    api::oauth::SlackOAuthV2AccessResponse,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{error, info};

// トークンストレージ
#[derive(Default)]
pub struct TokenStorage {
    tokens: HashMap<String, SlackOAuthV2AccessResponse>,
}

impl TokenStorage {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
        }
    }


    pub fn store_token(&mut self, team_id: String, token: SlackOAuthV2AccessResponse) {
        self.tokens.insert(team_id, token);
    }

    pub fn get_token(&self, team_id: &str) -> Option<&SlackOAuthV2AccessResponse> {
        self.tokens.get(team_id)
    }
}

// OAuth設定
#[derive(Clone)]
pub struct OAuthConfig {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    token_storage: Arc<RwLock<TokenStorage>>,
}

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

// OAuthルーターの設定
pub fn oauth_router(config: OAuthConfig) -> Router {
    Router::new()
        .route("/oauth/start", get(oauth_start))
        .route("/oauth/callback", get(oauth_callback))
        .with_state(config)
}

// OAuth開始ハンドラー
async fn oauth_start(
    State(config): State<OAuthConfig>,
) -> Response {
    let scopes = vec![
        "channels:history",
        "chat:write",
        "commands",
    ];

    let auth_url = format!(
        "https://slack.com/oauth/v2/authorize?client_id={}&scope={}",
        config.client_id,
        scopes.join(",")
    );
    
    info!("OAuth開始: {}", auth_url);
    axum::response::Redirect::temporary(&auth_url).into_response()
}

// OAuthコールバックハンドラー
async fn oauth_callback(
    State(config): State<OAuthConfig>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    if let Some(code) = params.get("code") {
        info!("OAuth認証コードを受信: {}", code);
        
        
        let client = SlackClient::new(SlackClientHyperConnector::new());
        
        // アクセストークンの取得
        let result = client
            .oauth_v2_access(
                &SlackOAuthV2AccessRequest::new(
                    &config.client_id,
                    &config.client_secret,
                    code,
                    Some(&config.redirect_uri),
                )
            )
            .await;

        match result {
            Ok(token_response) => {
                if let Some(team) = &token_response.team {
                    let mut storage = config.token_storage.write().await;
                    storage.store_token(team.id.clone(), token_response);
                    info!("チーム {} のトークンを保存しました", team.id);
                    "OAuth認証成功！".into_response()
                } else {
                    error!("チーム情報が見つかりません");
                    "OAuth認証エラー: チーム情報が見つかりません".into_response()
                }
            }
            Err(e) => {
                error!("OAuth認証エラー: {}", e);
                format!("OAuth認証エラー: {}", e).into_response()
            }
        }
    } else {
        "認証コードが見つかりません".into_response()
    }
}

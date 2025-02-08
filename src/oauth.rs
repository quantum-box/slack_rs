//! OAuth関連の型定義

use std::sync::Arc;
use tokio::sync::RwLock;

/// OAuth設定
#[derive(Clone)]
pub struct OAuthConfig {
    /// トークンストレージ
    pub token_storage: Arc<RwLock<dyn TokenStorage>>,
}

/// トークンストレージトレイト
pub trait TokenStorage: Send + Sync {
    /// トークンを取得
    fn get_token(&self, team_id: &str) -> Option<TokenResponse>;
}

/// トークンレスポンス
#[derive(Clone)]
pub struct TokenResponse {
    /// アクセストークン
    pub access_token: String,
}

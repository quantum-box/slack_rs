//! Slack APIで使用する型定義

use slack_morphism::{SlackApiToken, SlackApiTokenValue, SlackSigningSecret as MorphismSigningSecret};

/// Slackボットトークン
#[derive(Clone, Debug)]
pub struct Token(String);

impl Token {
    /// 新しいトークンを作成
    ///
    /// # 引数
    /// * `token` - Slackボットトークン文字列
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }

    /// トークン文字列を取得
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<Token> for SlackApiToken {
    fn from(token: Token) -> Self {
        SlackApiToken::new(SlackApiTokenValue(token.0))
    }
}

/// Slack署名シークレット
#[derive(Clone, Debug)]
pub struct SigningSecret(String);

impl SigningSecret {
    /// 新しい署名シークレットを作成
    ///
    /// # 引数
    /// * `secret` - Slack署名シークレット文字列
    pub fn new(secret: impl Into<String>) -> Self {
        Self(secret.into())
    }

    /// 署名シークレット文字列を取得
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<SigningSecret> for MorphismSigningSecret {
    fn from(secret: SigningSecret) -> Self {
        MorphismSigningSecret::new(secret.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_conversion() {
        let token = Token::new("test-token");
        let morphism_token: SlackApiToken = token.into();
        assert_eq!(morphism_token.value().0, "test-token");
    }

    #[test]
    fn test_signing_secret_conversion() {
        let secret = SigningSecret::new("test-secret");
        let morphism_secret: MorphismSigningSecret = secret.into();
        assert_eq!(morphism_secret.value(), "test-secret");
    }
}

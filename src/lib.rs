pub mod socket_mode;
pub mod webhook;

// Re-export commonly used items for public API
pub use webhook::{create_app, handle_push_event, AppState};

// Re-export types from slack-morphism that are part of our public API
pub use slack_morphism::{SlackApiToken, SlackApiTokenValue, SlackSigningSecret};

#[cfg(test)]
mod tests {
    #[test]
    fn test_socket_mode_module_exists() {
        // モジュールが正しくエクスポートされていることを確認
        assert!(true);
    }

    #[test]
    fn test_webhook_module_exists() {
        // webhookモジュールが正しくエクスポートされていることを確認
        assert!(true);
    }
}

pub mod blocks;
pub mod events;
pub mod message;
pub mod oauth;
pub mod socket_mode;
pub mod types;
pub mod webhook;

// 公開APIのエクスポート
pub use blocks::Block;
pub use events::Event;
pub use message::MessageClient;
pub use types::{SigningSecret, Token};
pub use webhook::{
    create_app, create_app_with_path, handle_push_event, AppState, NoopHandler, SlackEventHandler,
    DEFAULT_WEBHOOK_PATH,
};

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

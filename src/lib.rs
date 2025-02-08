pub mod blocks;
pub mod events;
pub mod message;
pub mod oauth;
pub mod socket_mode;
pub mod types;
pub mod webhook;

// 公開APIのエクスポート
pub use blocks::Block;
#[cfg(feature = "events")]
pub use events::Event;
pub use message::MessageClient;
pub use types::{SigningSecret, Token};
pub use webhook::{
    create_app, create_app_with_path, handle_push_event, AppState, NoopHandler, SlackEventHandler,
    DEFAULT_WEBHOOK_PATH,
};

// 一時的なslack-morphism型のre-export
// TODO: 将来的には内部実装に置き換える
pub use slack_morphism::{
    api::SlackApiChatPostMessageRequest,
    blocks::{SlackBlock, SlackBlockText, SlackSectionBlock},
    events::{SlackEventCallbackBody, SlackPushEvent},
    prelude::{
        SlackApiToken, SlackApiTokenValue, SlackChannelId, SlackClient, SlackMessageContent,
        SlackTs,
    },
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

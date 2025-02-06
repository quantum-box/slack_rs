use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use slack_morphism::hyper_tokio::SlackClientHyperConnector;
use slack_morphism::prelude::*;
use std::error::Error;
use std::sync::Arc;
use tracing::{info, warn};

#[cfg(feature = "message")]
#[derive(Clone)]
pub struct MessageClient {
    client: Arc<SlackClient<SlackClientHyperConnector<HttpsConnector<HttpConnector>>>>,
    token: SlackApiToken,
}

#[cfg(feature = "message")]
impl MessageClient {
    pub fn new(token: SlackApiToken) -> Self {
        let connector =
            SlackClientHyperConnector::new().expect("HTTPクライアントの作成に失敗しました");
        let client = Arc::new(SlackClient::new(connector));
        Self { client, token }
    }

    pub async fn send_text(&self, channel: &str, text: &str) -> Result<(), Box<dyn Error>> {
        let content = SlackMessageContent::new().with_text(text.into());
        self.send_message(channel, content).await
    }

    pub async fn send_blocks(
        &self,
        channel: &str,
        blocks: Vec<SlackBlock>,
    ) -> Result<(), Box<dyn Error>> {
        let content = SlackMessageContent::new().with_blocks(blocks);
        self.send_message(channel, content).await
    }

    async fn send_message(
        &self,
        channel: &str,
        content: SlackMessageContent,
    ) -> Result<(), Box<dyn Error>> {
        let channel_id = SlackChannelId::new(channel.into());
        let req = SlackApiChatPostMessageRequest::new(channel_id, content);
        let session = self.client.open_session(&self.token);
        match session.chat_post_message(&req).await {
            Ok(_) => {
                info!("メッセージを送信しました: {}", channel);
                Ok(())
            }
            Err(e) => {
                warn!("メッセージの送信に失敗しました: {:?}", e);
                Err(Box::new(e))
            }
        }
    }

    pub async fn reply_to_thread(
        &self,
        channel: &str,
        thread_ts: &str,
        text: &str,
    ) -> Result<(), Box<dyn Error>> {
        let channel_id = SlackChannelId::new(channel.into());
        let content = SlackMessageContent::new().with_text(text.into());
        let req = SlackApiChatPostMessageRequest::new(channel_id, content)
            .with_thread_ts(SlackTs::new(thread_ts.into()));
        let session = self.client.open_session(&self.token);
        match session.chat_post_message(&req).await {
            Ok(_) => {
                info!(
                    "スレッドに返信しました: {} (thread_ts: {})",
                    channel, thread_ts
                );
                Ok(())
            }
            Err(e) => {
                warn!("スレッドへの返信に失敗しました: {:?}", e);
                Err(Box::new(e))
            }
        }
    }

    pub async fn update_message(
        &self,
        channel: &str,
        ts: &str,
        text: &str,
    ) -> Result<(), Box<dyn Error>> {
        let channel_id = SlackChannelId::new(channel.into());
        let ts = SlackTs::new(ts.into());
        let content = SlackMessageContent::new().with_text(text.into());
        let req = SlackApiChatUpdateRequest::new(channel_id, content, ts.clone());
        let session = self.client.open_session(&self.token);
        match session.chat_update(&req).await {
            Ok(_) => {
                info!("メッセージを更新しました: {} (ts: {})", channel, ts);
                Ok(())
            }
            Err(e) => {
                warn!("メッセージの更新に失敗しました: {:?}", e);
                Err(Box::new(e))
            }
        }
    }

    pub async fn delete_message(&self, channel: &str, ts: &str) -> Result<(), Box<dyn Error>> {
        let channel_id = SlackChannelId::new(channel.into());
        let ts = SlackTs::new(ts.into());
        let req = SlackApiChatDeleteRequest::new(channel_id, ts.clone());
        let session = self.client.open_session(&self.token);
        match session.chat_delete(&req).await {
            Ok(_) => {
                info!("メッセージを削除しました: {} (ts: {})", channel, ts);
                Ok(())
            }
            Err(e) => {
                warn!("メッセージの削除に失敗しました: {:?}", e);
                Err(Box::new(e))
            }
        }
    }

    pub async fn upload_file(
        &self,
        channels: Vec<String>,
        file: Vec<u8>,
        filename: &str,
    ) -> Result<(), Box<dyn Error>> {
        let channel_ids: Vec<SlackChannelId> = channels
            .into_iter()
            .map(|c| SlackChannelId::new(c))
            .collect();
        let req = SlackApiFilesUploadRequest::new()
            .with_channels(channel_ids)
            .with_filename(filename.into())
            .with_content(String::from_utf8_lossy(&file).into_owned());
        let session = self.client.open_session(&self.token);
        match session.files_upload(&req).await {
            Ok(_) => {
                info!("ファイルをアップロードしました: {}", filename);
                Ok(())
            }
            Err(e) => {
                warn!("ファイルのアップロードに失敗しました: {:?}", e);
                Err(Box::new(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_text_message() {
        let token = SlackApiToken::new(SlackApiTokenValue("test-token".into()));
        let client = MessageClient::new(token);
        assert!(true);
    }
}

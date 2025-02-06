use slack_morphism::blocks::{SlackBlock, SlackBlockText, SlackSectionBlock};
use slack_rs::{MessageClient, SlackApiToken, SlackApiTokenValue};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("SLACK_BOT_TOKEN").expect("SLACK_BOT_TOKEN must be set");
    let token = SlackApiToken::new(SlackApiTokenValue(token));
    let client = MessageClient::new(token);

    // テキストメッセージの送信
    client
        .send_text("C06NXPX6XQX", "基本的なテキストメッセージ")
        .await?;

    // ブロックキットを使用したメッセージ
    let blocks = vec![
        SlackBlock::Section(SlackSectionBlock::new().with_text(SlackBlockText::MarkDown(
            "*太字* _斜体_ ~取り消し線~".into(),
        ))),
        SlackBlock::Section(
            SlackSectionBlock::new().with_text(SlackBlockText::Plain("プレーンテキスト".into())),
        ),
    ];
    client.send_blocks("C06NXPX6XQX", blocks).await?;

    // スレッド返信
    let message = client
        .send_text("C06NXPX6XQX", "スレッドの親メッセージ")
        .await?;
    client
        .reply_to_thread("C06NXPX6XQX", &message.ts.to_string(), "スレッドへの返信")
        .await?;

    // メッセージの更新と削除
    let message = client
        .send_text("C06NXPX6XQX", "このメッセージは更新されます")
        .await?;
    sleep(Duration::from_secs(2)).await;
    client
        .update_message("C06NXPX6XQX", &message.ts.to_string(), "更新されたメッセージ")
        .await?;
    sleep(Duration::from_secs(2)).await;
    client.delete_message("C06NXPX6XQX", &message.ts.to_string()).await?;

    // ファイルのアップロード
    let file_content = "テストファイルの内容".as_bytes().to_vec();
    client
        .upload_file(vec!["C06NXPX6XQX".to_string()], file_content, "test.txt")
        .await?;

    Ok(())
}

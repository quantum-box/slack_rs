use slack_morphism::blocks::{SlackBlock, SlackBlockElement, SlackBlockText};
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
        .send_text("#general", "基本的なテキストメッセージ")
        .await?;

    // ブロックキットを使用したメッセージ
    let blocks = vec![
        SlackBlock::Section(SlackBlockElement::new().with_text(SlackBlockText::Markdown(
            "*太字* _斜体_ ~取り消し線~".into(),
        ))),
        SlackBlock::Section(
            SlackBlockElement::new()
                .with_text(SlackBlockText::PlainText("プレーンテキスト".into())),
        ),
    ];
    client.send_blocks("#general", blocks).await?;

    // スレッド返信
    let message = client
        .send_text("#general", "スレッドの親メッセージ")
        .await?;
    client
        .reply_to_thread("#general", &message.ts, "スレッドへの返信")
        .await?;

    // メッセージの更新と削除
    let message = client
        .send_text("#general", "このメッセージは更新されます")
        .await?;
    sleep(Duration::from_secs(2)).await;
    client
        .update_message("#general", &message.ts, "更新されたメッセージ")
        .await?;
    sleep(Duration::from_secs(2)).await;
    client.delete_message("#general", &message.ts).await?;

    // ファイルのアップロード
    let file_content = "テストファイルの内容".as_bytes().to_vec();
    client
        .upload_file(vec!["#general".to_string()], file_content, "test.txt")
        .await?;

    Ok(())
}

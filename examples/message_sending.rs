use slack_rs::{Block, MessageClient, Token};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("SLACK_BOT_TOKEN").expect("SLACK_BOT_TOKEN must be set");
    let token = Token::new(token);
    let client = MessageClient::new(token);

    // テキストメッセージの送信
    client
        .send_text("C087D6X8NM9", "基本的なテキストメッセージ")
        .await?;

    // ブロックキットを使用したメッセージ
    let blocks = vec![
        Block::Section {
            text: "*太字* _斜体_ ~取り消し線~".to_string(),
        },
        Block::Section {
            text: "プレーンテキスト".to_string(),
        },
    ];
    client.send_blocks("C087D6X8NM9", blocks).await?;

    // ブロックキットを使用したスレッド返信
    let reply_blocks = vec![
        Block::Section {
            text: "*スレッド返信* with _BlockKit_".to_string(),
        },
        Block::Section {
            text: "複数のブロックを使用した返信が可能です".to_string(),
        },
    ];
    client
        .reply_to_thread_with_blocks("C087D6X8NM9", "1234567890.123456", reply_blocks)
        .await?;

    // スレッド返信
    client
        .send_text("C087D6X8NM9", "スレッドの親メッセージ")
        .await?;
    client
        .reply_to_thread("C087D6X8NM9", "1234567890.123456", "スレッドへの返信")
        .await?;

    // メッセージの更新と削除
    client
        .send_text("C087D6X8NM9", "このメッセージは更新されます")
        .await?;
    sleep(Duration::from_secs(2)).await;
    client
        .update_message("C087D6X8NM9", "1234567890.123456", "更新されたメッセージ")
        .await?;
    sleep(Duration::from_secs(2)).await;
    client
        .delete_message("C087D6X8NM9", "1234567890.123456")
        .await?;

    // ファイルのアップロード
    let file_content = "テストファイルの内容".as_bytes().to_vec();
    client
        .upload_file(vec!["C087D6X8NM9".to_string()], file_content, "test.txt")
        .await?;

    Ok(())
}

use slack_rs::{Block, MessageClient, Token};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let token = std::env::var("SLACK_BOT_TOKEN").expect("SLACK_BOT_TOKEN must be set");
    let token = Token::new(token);
    let client = MessageClient::new(token);

    // テキストメッセージの送信
    let text_ts = client
        .send_text("C087D6X8NM9", "基本的なテキストメッセージ")
        .await?;
    println!("テキストメッセージのts: {}", text_ts);

    // ブロックキットを使用したメッセージ
    let blocks = vec![
        Block::Section {
            text: "*太字*\n_斜体_\n~取り消し線~".to_string(),
        },
        Block::Section {
            text: "1行目\n2行目\n3行目".to_string(),
        },
        Block::Section {
            text: "改行を含む\nリスト:\n• 項目1\n• 項目2\n• 項目3".to_string(),
        },
    ];
    let block_ts = client.send_blocks("C087D6X8NM9", blocks).await?;
    println!("ブロックメッセージのts: {}", block_ts);

    // ブロックキットを使用したスレッド返信
    let reply_blocks = vec![
        Block::Section {
            text: "*スレッド返信* with _BlockKit_".to_string(),
        },
        Block::Section {
            text: "複数のブロックを使用した返信が可能です".to_string(),
        },
    ];
    let reply_block_ts = client
        .reply_to_thread_with_blocks("C087D6X8NM9", &block_ts, reply_blocks)
        .await?;
    println!("ブロック返信のts: {}", reply_block_ts);

    // スレッド返信
    let parent_ts = client
        .send_text("C087D6X8NM9", "スレッドの親メッセージ")
        .await?;
    println!("親メッセージのts: {}", parent_ts);

    let reply_ts = client
        .reply_to_thread("C087D6X8NM9", &parent_ts, "スレッドへの返信")
        .await?;
    println!("返信メッセージのts: {}", reply_ts);

    // メッセージの更新と削除
    let update_target_ts = client
        .send_text("C087D6X8NM9", "このメッセージは更新されます")
        .await?;
    sleep(Duration::from_secs(2)).await;
    client
        .update_message("C087D6X8NM9", &update_target_ts, "更新されたメッセージ")
        .await?;
    sleep(Duration::from_secs(2)).await;
    client
        .delete_message("C087D6X8NM9", &update_target_ts)
        .await?;

    // ファイルのアップロード
    let file_content = "テストファイルの内容".as_bytes().to_vec();
    client
        .upload_file(vec!["C087D6X8NM9".to_string()], file_content, "test.txt")
        .await?;

    Ok(())
}

use slack_rs::{Block, MessageClient, Token};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("SLACK_BOT_TOKEN").expect("SLACK_BOT_TOKEN must be set");
    let token = Token::new(token);
    let client = MessageClient::new(token);

    // テキストメッセージの送信
    client.send_text("#general", "Hello, World!").await?;

    // ブロックキットを使用したメッセージ
    let blocks = vec![Block::Section {
        text: "*Bold* _italic_ ~strike~".to_string(),
    }];
    client.send_blocks("#general", blocks).await?;

    Ok(())
}

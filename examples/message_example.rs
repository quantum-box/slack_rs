use slack_morphism::blocks::{SlackBlock, SlackBlockText, SlackSectionBlock};
use slack_rs::{MessageClient, SlackApiToken, SlackApiTokenValue};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("SLACK_BOT_TOKEN").expect("SLACK_BOT_TOKEN must be set");
    let token = SlackApiToken::new(SlackApiTokenValue(token));
    let client = MessageClient::new(token);

    // テキストメッセージの送信
    client.send_text("#general", "Hello, World!").await?;

    // ブロックキットを使用したメッセージ
    let blocks = vec![SlackBlock::Section(
        SlackSectionBlock::new()
            .with_text(SlackBlockText::MarkDown("*Bold* _italic_ ~strike~".into())),
    )];
    client.send_blocks("#general", blocks).await?;

    Ok(())
}

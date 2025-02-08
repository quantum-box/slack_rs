//! Slackメッセージのブロック要素を定義するモジュール

use slack_morphism::blocks::{
    SlackBlock as MorphismBlock, SlackBlockText, SlackDividerBlock, SlackSectionBlock,
};

/// Slackメッセージのブロック要素
#[derive(Debug, Clone)]
pub enum Block {
    /// セクションブロック
    Section {
        /// セクションのテキスト
        text: String,
    },
    /// 区切り線
    Divider,
}

impl From<Block> for MorphismBlock {
    fn from(block: Block) -> Self {
        match block {
            Block::Section { text } => SlackSectionBlock::new()
                .with_text(SlackBlockText::Plain(text.into()))
                .into(),
            Block::Divider => SlackDividerBlock::new().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_block_conversion() {
        let block = Block::Section {
            text: "テストメッセージ".to_string(),
        };
        let _morphism_block: MorphismBlock = block.into();
    }

    #[test]
    fn test_divider_block_conversion() {
        let block = Block::Divider;
        let _morphism_block: MorphismBlock = block.into();
    }

    #[test]
    fn test_block_list_conversion() {
        let blocks = vec![
            Block::Section {
                text: "テスト1".to_string(),
            },
            Block::Divider,
            Block::Section {
                text: "テスト2".to_string(),
            },
        ];
        let _morphism_blocks: Vec<MorphismBlock> = blocks.into_iter().map(Into::into).collect();
    }
}

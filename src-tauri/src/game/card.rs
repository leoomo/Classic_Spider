use serde::{Deserialize, Serialize};

/// 花色枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Suit {
    Spade,   // 黑桃 ♠
    Heart,   // 红桃 ♥
    Diamond, // 方块 ♦
    Club,    // 梅花 ♣
}

impl Suit {
    /// 是否为红色花色
    pub fn is_red(self) -> bool {
        matches!(self, Suit::Heart | Suit::Diamond)
    }

    /// 获取花色符号
    pub fn symbol(self) -> &'static str {
        match self {
            Suit::Spade => "♠",
            Suit::Heart => "♥",
            Suit::Diamond => "♦",
            Suit::Club => "♣",
        }
    }

    /// 根据难度获取可用的花色列表
    pub fn for_difficulty(difficulty: u8) -> Vec<Self> {
        match difficulty {
            1 => vec![Suit::Spade; 8],  // 初级：8副黑桃
            2 => {
                // 中级：4副黑桃 + 4副红桃
                vec![
                    Suit::Spade, Suit::Heart,
                    Suit::Spade, Suit::Heart,
                    Suit::Spade, Suit::Heart,
                    Suit::Spade, Suit::Heart,
                ]
            }
            _ => {
                // 高级：各2副花色
                vec![
                    Suit::Spade, Suit::Spade,
                    Suit::Heart, Suit::Heart,
                    Suit::Diamond, Suit::Diamond,
                    Suit::Club, Suit::Club,
                ]
            }
        }
    }
}

/// 卡牌结构
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    /// 唯一标识符
    pub id: u32,
    /// 花色
    pub suit: Suit,
    /// 点数 (1=A, 11=J, 12=Q, 13=K)
    pub value: u8,
    /// 是否正面朝上
    pub face_up: bool,
}

impl Card {
    /// 创建新卡牌
    pub fn new(id: u32, suit: Suit, value: u8) -> Self {
        Self {
            id,
            suit,
            value: value.clamp(1, 13),
            face_up: false,
        }
    }

    /// 获取点数显示文本
    pub fn display_value(self) -> String {
        match self.value {
            1 => "A".to_string(),
            11 => "J".to_string(),
            12 => "Q".to_string(),
            13 => "K".to_string(),
            v => v.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suit_color() {
        assert!(!Suit::Spade.is_red());
        assert!(Suit::Heart.is_red());
        assert!(Suit::Diamond.is_red());
        assert!(!Suit::Club.is_red());
    }

    #[test]
    fn test_suit_symbol() {
        assert_eq!(Suit::Spade.symbol(), "♠");
        assert_eq!(Suit::Heart.symbol(), "♥");
    }

    #[test]
    fn test_card_display() {
        let ace = Card::new(0, Suit::Spade, 1);
        assert_eq!(ace.display_value(), "A");

        let king = Card::new(1, Suit::Heart, 13);
        assert_eq!(king.display_value(), "K");
    }
}

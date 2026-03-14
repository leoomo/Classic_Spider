use super::card::Card;

#[cfg(test)]
use super::card::Suit;

#[cfg(test)]
use super::state::GameState;

/// 检查序列是否可以整体拖拽
///
/// 条件：
/// 1. 所有牌必须正面朝上
/// 2. 必须是同花色
/// 3. 必须是连续递减序列 (K->A)
pub fn can_drag_sequence(cards: &[Card]) -> bool {
    if cards.is_empty() {
        return false;
    }

    // 检查是否全部翻开
    if !cards.iter().all(|c| c.face_up) {
        return false;
    }

    // 单张牌总是可以拖拽
    if cards.len() == 1 {
        return true;
    }

    // 检查同花色且连续递减
    let first_suit = cards[0].suit;
    for window in cards.windows(2) {
        let current = &window[0];
        let next = &window[1];

        // 检查同花色
        if current.suit != first_suit || next.suit != first_suit {
            return false;
        }

        // 检查连续递减 (当前比下一个大 1)
        if current.value != next.value + 1 {
            return false;
        }
    }

    true
}

/// 检查是否可以放置到目标列
///
/// 条件：
/// 1. 目标列为空时，任何序列都可以放置
/// 2. 目标列非空时，被拖拽序列的第一张牌，比目标列最后一张牌小 1
pub fn can_place_on(moving_cards: &[Card], target_column: &[Card]) -> bool {
    if moving_cards.is_empty() {
        return false;
    }

    // 空列可以放任何牌
    if target_column.is_empty() {
        return true;
    }

    let target_card = target_column.last().unwrap();
    let first_moving = &moving_cards[0];

    // 只需要数值比目标小 1
    target_card.value == first_moving.value + 1
}

/// 检查是否形成完整的 K-A 序列
///
/// 条件：
/// 1. 必须是 13 张牌
/// 2. 必须是同花色
/// 3. 必须是 K 到 A 的完整序列
pub fn is_complete_sequence(cards: &[Card]) -> bool {
    if cards.len() != 13 {
        return false;
    }

    let suit = cards[0].suit;

    for (i, card) in cards.iter().enumerate() {
        // 检查花色
        if card.suit != suit {
            return false;
        }
        // 检查数值：K=13, Q=12, ..., A=1
        if card.value != (13 - i) as u8 {
            return false;
        }
    }

    true
}

/// 查找可用的提示（合法移动)
///
/// 返回: Option<(from_col, start_idx, to_col)>
/// - from_col: 源列索引
/// - start_idx: 在源列中的起始位置
/// - to_col: 目标列索引
pub fn find_hint(columns: &[Vec<Card>]) -> Option<(usize, usize, usize)> {
    // 从每一列寻找可移动的序列
    for (from_col, column) in columns.iter().enumerate() {
        // 从最底部开始尝试最长的序列
        for start_idx in 0..column.len() {
            let sequence = &column[start_idx..];

            if !can_drag_sequence(sequence) {
                continue;
            }

            // 寻找可以放置的目标列
            for (to_col, target) in columns.iter().enumerate() {
                if from_col == to_col {
                    continue;
                }

                if can_place_on(sequence, target) {
                    // 优先返回有意义的移动
                    // 避免将整列移到空列
                    if target.is_empty() && sequence.len() == column.len() {
                        continue;
                    }
                    return Some((from_col, start_idx, to_col));
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_card(id: u32, suit: Suit, value: u8, face_up: bool) -> Card {
        Card { id, suit, value, face_up }
    }

    #[test]
    fn test_can_drag_single_card() {
        let card = create_card(0, Suit::Spade, 5, true);
        assert!(can_drag_sequence(&[card]));
    }

    #[test]
    fn test_can_drag_face_down_card() {
        let card = create_card(0, Suit::Spade, 5, false);
        assert!(!can_drag_sequence(&[card]));
    }

    #[test]
    fn test_can_drag_valid_sequence() {
        let cards = vec![
            create_card(0, Suit::Spade, 5, true),
            create_card(1, Suit::Spade, 4, true),
            create_card(2, Suit::Spade, 3, true),
        ];
        assert!(can_drag_sequence(&cards));
    }

    #[test]
    fn test_cannot_drag_mixed_suits() {
        let cards = vec![
            create_card(0, Suit::Spade, 5, true),
            create_card(1, Suit::Heart, 4, true),
        ];
        assert!(!can_drag_sequence(&cards));
    }

    #[test]
    fn test_cannot_drag_non_sequential() {
        let cards = vec![
            create_card(0, Suit::Spade, 5, true),
            create_card(1, Suit::Spade, 3, true), // Should be 4
        ];
        assert!(!can_drag_sequence(&cards));
    }

    #[test]
    fn test_can_place_on_empty_column() {
        let cards = vec![create_card(0, Suit::Spade, 5, true)];
        assert!(can_place_on(&cards, &[]));
    }

    #[test]
    fn test_can_place_on_higher_card() {
        let cards = vec![create_card(0, Suit::Spade, 5, true)];
        let target = vec![create_card(1, Suit::Heart, 6, true)];
        assert!(can_place_on(&cards, &target));
    }

    #[test]
    fn test_cannot_place_on_lower_card() {
        let cards = vec![create_card(0, Suit::Spade, 5, true)];
        let target = vec![create_card(1, Suit::Heart, 4, true)];
        assert!(!can_place_on(&cards, &target));
    }

    #[test]
    fn test_cannot_place_on_same_value() {
        let cards = vec![create_card(0, Suit::Spade, 5, true)];
        let target = vec![create_card(1, Suit::Heart, 5, true)];
        assert!(!can_place_on(&cards, &target));
    }

    #[test]
    fn test_is_complete_sequence_valid() {
        let mut cards = Vec::new();
        for i in 0..13 {
            cards.push(create_card(i as u32, Suit::Spade, 13 - i as u8, true));
        }
        assert!(is_complete_sequence(&cards));
    }

    #[test]
    fn test_is_complete_sequence_wrong_length() {
        let cards = vec![
            create_card(0, Suit::Spade, 13, true),
            create_card(1, Suit::Spade, 12, true),
        ];
        assert!(!is_complete_sequence(&cards));
    }

    #[test]
    fn test_is_complete_sequence_mixed_suits() {
        let mut cards = Vec::new();
        for i in 0..13 {
            let suit = if i % 2 == 0 { Suit::Spade } else { Suit::Heart };
            cards.push(create_card(i as u32, suit, 13 - i as u8, true));
        }
        assert!(!is_complete_sequence(&cards));
    }

    #[test]
    fn test_is_complete_sequence_wrong_order() {
        let mut cards = Vec::new();
        for i in 0..13 {
            cards.push(create_card(i as u32, Suit::Spade, i as u8 + 1, true));
        }
        assert!(!is_complete_sequence(&cards));
    }
}

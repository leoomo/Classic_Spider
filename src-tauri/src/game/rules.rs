use super::card::{Card, Suit};

use super::state::GameState;
use serde::{Deserialize, Serialize};

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

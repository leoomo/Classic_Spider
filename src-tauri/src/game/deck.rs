use super::card::{Card, Suit};
use rand::seq::SliceRandom;
use rand::thread_rng;

/// 生成一副完整的蜘蛛纸牌牌组（104张）
pub fn generate_deck(difficulty: u8) -> Vec<Card> {
    let mut deck = Vec::with_capacity(104);
    let suits = Suit::for_difficulty(difficulty);
    let mut id_counter = 0u32;

    // 每种花色配置生成 2 副牌（每种花色 26 张，共 104 张）
    for &suit in &suits {
        for _ in 0..2 {
            for value in 1..=13 {
                deck.push(Card::new(id_counter, suit, value));
                id_counter += 1;
            }
        }
    }

    // 洗牌
    let mut rng = thread_rng();
    deck.shuffle(&mut rng);

    deck
}

/// 将牌组分配到初始布局
/// 返回 (10列牌堆, 发牌堆)
pub fn deal_initial_cards(deck: &mut Vec<Card>) -> (Vec<Vec<Card>>, Vec<Card>) {
    let mut columns = vec![Vec::new(); 10];

    // 前 4 列 6 张，后 6 列 5 张，共 54 张
    let column_sizes = [6, 6, 6, 6, 5, 5, 5, 5, 5, 5];

    for (col_idx, &size) in column_sizes.iter().enumerate() {
        for i in 0..size {
            if let Some(mut card) = deck.pop() {
                // 每列最后一张翻开
                if i == size - 1 {
                    card.face_up = true;
                }
                columns[col_idx].push(card);
            }
        }
    }

    // 剩余 50 张作为发牌堆
    let stock = std::mem::take(deck);

    (columns, stock)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_deck_difficulty_1() {
        // Difficulty 1: 8 spade decks × 2 × 13 = 208 cards
        let deck = generate_deck(1);
        assert_eq!(deck.len(), 208);
    }

    #[test]
    fn test_generate_deck_difficulty_2() {
        // Difficulty 2: 8 mixed decks × 2 × 13 = 208 cards
        let deck = generate_deck(2);
        assert_eq!(deck.len(), 208);
    }

    #[test]
    fn test_generate_deck_difficulty_3() {
        // Difficulty 3: 8 mixed decks × 2 × 13 = 208 cards
        let deck = generate_deck(3);
        assert_eq!(deck.len(), 208);
    }

    #[test]
    fn test_deal_initial_cards() {
        let mut deck = generate_deck(1);
        let (columns, stock) = deal_initial_cards(&mut deck);

        // 检查列数
        assert_eq!(columns.len(), 10);

        // 检查总牌数 (54 in columns, rest in stock)
        let column_cards: usize = columns.iter().map(|c| c.len()).sum();
        assert_eq!(column_cards, 54);
        assert_eq!(stock.len(), 154); // 208 - 54 = 154

        // 检查前4列有6张，后6列有5张
        for (i, col) in columns.iter().enumerate() {
            let expected = if i < 4 { 6 } else { 5 };
            assert_eq!(col.len(), expected);
        }
    }
}

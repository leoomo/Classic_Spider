use super::card::{Card, Suit};
use rand::seq::SliceRandom;
use rand::thread_rng;

/// 生成一副完整的蜘蛛纸牌牌组（104张）
pub fn generate_deck(difficulty: u8) -> Vec<Card> {
    let mut deck = Vec::with_capacity(104);
    let suits = Suit::for_difficulty(difficulty);
    let mut id_counter = 0u32;

    // 每种花色配置生成 1 副牌（8个花色配置 × 13张 = 104张）
    for &suit in &suits {
        for value in 1..=13 {
            deck.push(Card::new(id_counter, suit, value));
            id_counter += 1;
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
        // Difficulty 1: 8 spade decks × 13 = 104 cards
        let deck = generate_deck(1);
        assert_eq!(deck.len(), 104);
    }

    #[test]
    fn test_generate_deck_difficulty_2() {
        // Difficulty 2: 8 mixed decks × 13 = 104 cards
        let deck = generate_deck(2);
        assert_eq!(deck.len(), 104);
    }

    #[test]
    fn test_generate_deck_difficulty_3() {
        // Difficulty 3: 8 mixed decks × 13 = 104 cards
        let deck = generate_deck(3);
        assert_eq!(deck.len(), 104);
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
        assert_eq!(stock.len(), 50); // 104 - 54 = 50

        // 检查前4列有6张，后6列有5张
        for (i, col) in columns.iter().enumerate() {
            let expected = if i < 4 { 6 } else { 5 };
            assert_eq!(col.len(), expected);
        }
    }

    /// 运行1000次模拟洗牌，验证牌组永远正确
    #[test]
    fn test_shuffle_1000_times_consistency() {
        use std::collections::HashMap;

        for iteration in 0..1000 {
            for difficulty in 1..=3 {
                let deck = generate_deck(difficulty);

                // 1. 验证总牌数永远是 104 张
                assert_eq!(
                    deck.len(),
                    104,
                    "第{}次迭代，难度{}：牌组应有104张，实际{}张",
                    iteration,
                    difficulty,
                    deck.len()
                );

                // 2. 统计每个点数的牌数量
                let mut value_counts: HashMap<u8, usize> = HashMap::new();
                for card in &deck {
                    *value_counts.entry(card.value).or_insert(0) += 1;
                }

                // 3. 验证每个点数都有8张 (8副牌 × 1)
                // 8个花色配置，每种点数有 8 × 1 = 8 张
                for value in 1..=13 {
                    let count = value_counts.get(&value).unwrap_or(&0);
                    assert_eq!(
                        *count,
                        8,
                        "第{}次迭代，难度{}：点数{}应有8张，实际{}张",
                        iteration,
                        difficulty,
                        value,
                        count
                    );
                }

                // 4. 特别验证 A (value=1) 永远不会多出
                let ace_count = value_counts.get(&1).unwrap_or(&0);
                assert_eq!(
                    *ace_count,
                    8,
                    "第{}次迭代，难度{}：A应有8张，实际{}张 - 严重错误！",
                    iteration,
                    difficulty,
                    ace_count
                );
            }
        }
    }

    /// 验证所有卡牌ID唯一
    #[test]
    fn test_all_card_ids_unique() {
        use std::collections::HashSet;

        for difficulty in 1..=3 {
            let deck = generate_deck(difficulty);
            let ids: HashSet<u32> = deck.iter().map(|c| c.id).collect();

            assert_eq!(
                ids.len(),
                deck.len(),
                "难度{}：存在重复的卡牌ID",
                difficulty
            );
        }
    }

    /// 验证洗牌后牌组确实被打乱（非确定性测试）
    #[test]
    fn test_shuffle_randomness() {
        let deck1 = generate_deck(1);
        let deck2 = generate_deck(1);

        // 两次洗牌结果应该不同（极低概率相同）
        let first_10_match = deck1.iter().take(10).zip(deck2.iter().take(10))
            .filter(|(a, b)| a.id == b.id)
            .count();

        // 前10张牌完全相同的概率极低
        assert!(
            first_10_match < 10,
            "洗牌随机性测试失败：两次洗牌前10张完全相同"
        );
    }
}

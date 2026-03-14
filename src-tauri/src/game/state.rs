use super::card::Card;
use super::deck::{deal_initial_cards, generate_deck};
use super::history::History;
use super::rules::{can_drag_sequence, can_place_on, find_hint, is_complete_sequence};
use serde::{Deserialize, Serialize};

/// 游戏状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// 10 列牌堆
    pub columns: Vec<Vec<Card>>,
    /// 发牌堆（剩余 50 张）
    pub stock: Vec<Card>,
    /// 完成区（已收集的完整序列数量）
    pub completed: u8,
    /// 当前分数
    pub score: i32,
    /// 移动次数
    pub moves: u32,
    /// 难度 (1-3)
    pub difficulty: u8,
    /// 游戏是否结束
    pub game_over: bool,
    /// 是否获胜
    pub won: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            columns: vec![Vec::new(); 10],
            stock: Vec::new(),
            completed: 0,
            score: 500,
            moves: 0,
            difficulty: 1,
            game_over: false,
            won: false,
        }
    }
}

impl GameState {
    /// 创建新游戏
    pub fn new(difficulty: u8) -> (Self, History) {
        let mut deck = generate_deck(difficulty);
        let (columns, stock) = deal_initial_cards(&mut deck);

        let state = Self {
            columns,
            stock,
            completed: 0,
            score: 500,
            moves: 0,
            difficulty: difficulty.clamp(1, 3),
            game_over: false,
            won: false,
        };

        let history = History::new(state.clone());
        (state, history)
    }

    /// 获取剩余发牌次数
    #[allow(dead_code)]
    pub fn remaining_deals(&self) -> u8 {
        (self.stock.len() / 10) as u8
    }

    /// 检查是否可以发牌
    #[allow(dead_code)]
    pub fn can_deal(&self) -> bool {
        !self.stock.is_empty() && self.columns.iter().all(|col| !col.is_empty())
    }

    /// 移动卡牌
    pub fn move_cards(&mut self, from_col: usize, start_idx: usize, to_col: usize) -> Result<(), String> {
        // 验证索引
        if from_col >= 10 || to_col >= 10 {
            return Err("Invalid column index".to_string());
        }

        let from_column = &self.columns[from_col];
        if start_idx >= from_column.len() {
            return Err("Invalid start index".to_string());
        }

        // 获取要移动的序列
        let moving_cards = &from_column[start_idx..];

        // 验证拖拽合法性
        if !can_drag_sequence(moving_cards) {
            return Err("Cannot drag this sequence".to_string());
        }

        // 验证目标合法性
        if !can_place_on(moving_cards, &self.columns[to_col]) {
            return Err("Invalid destination".to_string());
        }

        // 执行移动
        let moving = self.columns[from_col].split_off(start_idx);
        self.columns[to_col].extend(moving);

        // 翻开原列最后一张牌
        if let Some(card) = self.columns[from_col].last_mut() {
            card.face_up = true;
        }

        // 更新分数和步数
        self.moves += 1;
        self.score -= 1;

        // 检查是否完成序列
        self.check_complete_sequence(to_col);

        // 检查是否获胜
        if self.completed == 8 {
            self.won = true;
            self.game_over = true;
        }

        Ok(())
    }

    /// 检查并移除完整的 K-A 序列
    fn check_complete_sequence(&mut self, col_idx: usize) {
        let column = &self.columns[col_idx];
        if column.len() < 13 {
            return;
        }

        // 检查最后 13 张是否形成完整序列
        let start_idx = column.len() - 13;
        let sequence = &column[start_idx..];

        if is_complete_sequence(sequence) {
            // 移除并更新完成计数
            self.columns[col_idx].truncate(start_idx);
            self.completed += 1;
            self.score += 100;

            // 翻开新的最后一张牌
            if let Some(card) = self.columns[col_idx].last_mut() {
                card.face_up = true;
            }
        }
    }

    /// 发牌
    pub fn deal(&mut self) -> Result<(), String> {
        // 检查是否有牌可发
        if self.stock.is_empty() {
            return Err("No cards left to deal".to_string());
        }

        // 检查所有列是否都有牌
        for (i, col) in self.columns.iter().enumerate() {
            if col.is_empty() {
                return Err(format!("Column {} is empty, cannot deal", i + 1).to_string());
            }
        }

        // 向每列发一张牌
        for col in &mut self.columns {
            if let Some(mut card) = self.stock.pop() {
                card.face_up = true;
                col.push(card);
            }
        }

        // 检查所有列是否形成完整序列
        for i in 0..10 {
            self.check_complete_sequence(i);
        }

        // 更新步数
        self.moves += 1;

        Ok(())
    }

    /// 获取提示
    pub fn get_hint(&self) -> Option<(usize, usize, usize)> {
        find_hint(&self.columns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let (game, _history) = GameState::new(1);

        // 检查初始分数
        assert_eq!(game.score, 500);

        // 检查牌数 (104 total: 54 in columns, 50 in stock)
        let column_cards: usize = game.columns.iter().map(|c| c.len()).sum();
        assert_eq!(column_cards, 54);
        assert_eq!(game.stock.len(), 50);

        // 检查剩余发牌次数 (50 / 10 = 5 deals)
        assert_eq!(game.remaining_deals(), 5);
    }

    #[test]
    fn test_new_game_difficulty_2() {
        let (game, _history) = GameState::new(2);
        assert_eq!(game.difficulty, 2);
        assert_eq!(game.columns.len(), 10);
    }

    #[test]
    fn test_new_game_difficulty_clamped() {
        let (game, _history) = GameState::new(5);
        assert_eq!(game.difficulty, 3); // Clamped to max 3
    }

    #[test]
    fn test_move_cards_invalid_column() {
        let (mut game, _history) = GameState::new(1);
        let result = game.move_cards(10, 0, 0); // Invalid column index
        assert!(result.is_err());
    }

    #[test]
    fn test_move_cards_invalid_start_index() {
        let (mut game, _history) = GameState::new(1);
        let result = game.move_cards(0, 100, 1); // Invalid start index
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_deal_with_empty_column() {
        let (mut game, _history) = GameState::new(1);
        // Clear one column
        game.columns[0].clear();
        let result = game.deal();
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_deal_with_empty_stock() {
        let (mut game, _history) = GameState::new(1);
        game.stock.clear();
        let result = game.deal();
        assert!(result.is_err());
    }

    #[test]
    fn test_can_deal() {
        let game = GameState::new(1).0;
        assert!(game.can_deal());
    }

    #[test]
    fn test_can_deal_returns_false_with_empty_column() {
        let mut game = GameState::new(1).0;
        game.columns[0].clear();
        assert!(!game.can_deal());
    }

    #[test]
    fn test_moves_increment() {
        let (mut game, _history) = GameState::new(1);
        let initial_moves = game.moves;
        // Find a valid move and execute it
        if let Some(hint) = game.get_hint() {
            let _ = game.move_cards(hint.0, hint.1, hint.2);
            assert_eq!(game.moves, initial_moves + 1);
        }
    }

    #[test]
    fn test_score_decrements_on_move() {
        let (mut game, _history) = GameState::new(1);
        let initial_score = game.score;
        if let Some(hint) = game.get_hint() {
            let _ = game.move_cards(hint.0, hint.1, hint.2);
            assert_eq!(game.score, initial_score - 1);
        }
    }
}

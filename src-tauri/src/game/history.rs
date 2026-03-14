use super::state::GameState;
use serde::{Deserialize, Serialize};

/// 历史记录管理（用于撤销/重做）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    /// 状态栈
    states: Vec<GameState>,
    /// 当前索引
    current_index: usize,
    /// 最大历史记录数
    max_size: usize,
}

impl History {
    pub fn new(initial_state: GameState) -> Self {
        Self {
            states: vec![initial_state],
            current_index: 0,
            max_size: 50,
        }
    }

    /// 保存新状态（会清除重做栈）
    pub fn push(&mut self, state: GameState) {
        println!("[History] push called: current_index={}, states_len={}", self.current_index, self.states.len());

        // 移除当前位置之后的所有状态
        self.states.truncate(self.current_index + 1);

        // 添加新状态
        self.states.push(state);

        // 如果超过最大大小，移除最早的状态
        if self.states.len() > self.max_size {
            self.states.remove(0);
        } else {
            self.current_index += 1;
        }

        println!("[History] after push: current_index={}, states_len={}", self.current_index, self.states.len());
    }

    /// 撤销
    pub fn undo(&mut self) -> Option<GameState> {
        println!("[History] undo called: current_index={}, states_len={}", self.current_index, self.states.len());
        if self.current_index > 0 {
            self.current_index -= 1;
            println!("[History] undo success: new current_index={}", self.current_index);
            Some(self.states[self.current_index].clone())
        } else {
            println!("[History] undo failed: already at beginning");
            None
        }
    }

    /// 重做
    pub fn redo(&mut self) -> Option<GameState> {
        if self.current_index < self.states.len() - 1 {
            self.current_index += 1;
            Some(self.states[self.current_index].clone())
        } else {
            None
        }
    }

    /// 是否可以撤销
    pub fn can_undo(&self) -> bool {
        println!("[History] can_undo: current_index={}, result={}", self.current_index, self.current_index > 0);
        self.current_index > 0
    }

    /// 是否可以重做
    pub fn can_redo(&self) -> bool {
        let result = self.current_index < self.states.len() - 1;
        println!("[History] can_redo: current_index={}, states_len={}, result={}", self.current_index, self.states.len(), result);
        result
    }

    /// 调试信息
    pub fn debug_info(&self) -> (usize, usize) {
        (self.current_index, self.states.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state(score: i32) -> GameState {
        let mut state = GameState::default();
        state.score = score;
        state
    }

    #[test]
    fn test_new_history() {
        let state = create_test_state(500);
        let history = History::new(state);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_push_and_undo() {
        let state1 = create_test_state(500);
        let mut history = History::new(state1);

        let state2 = create_test_state(499);
        history.push(state2);

        assert!(history.can_undo());
        assert!(!history.can_redo());

        let undone = history.undo();
        assert!(undone.is_some());
        assert_eq!(undone.unwrap().score, 500);
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn test_redo() {
        let state1 = create_test_state(500);
        let mut history = History::new(state1);

        let state2 = create_test_state(499);
        history.push(state2);

        history.undo();
        assert!(history.can_redo());

        let redone = history.redo();
        assert!(redone.is_some());
        assert_eq!(redone.unwrap().score, 499);
        assert!(!history.can_redo());
    }

    #[test]
    fn test_push_clears_redo_stack() {
        let state1 = create_test_state(500);
        let mut history = History::new(state1);

        let state2 = create_test_state(499);
        history.push(state2);

        let state3 = create_test_state(498);
        history.push(state3);

        history.undo(); // Back to state2
        history.undo(); // Back to state1

        let state4 = create_test_state(497);
        history.push(state4);

        // After push, redo stack should be cleared
        assert!(!history.can_redo());
        assert_eq!(history.states.len(), 2);
    }

    #[test]
    fn test_max_size() {
        let state = create_test_state(500);
        let mut history = History::new(state);

        // Push more than max_size states
        for i in 1..60 {
            let s = create_test_state(500 - i);
            history.push(s);
        }

        // Should not exceed max_size
        assert!(history.states.len() <= history.max_size);
    }
}

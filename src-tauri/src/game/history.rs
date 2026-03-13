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
    }

    /// 撤销
    pub fn undo(&mut self) -> Option<GameState> {
        if self.current_index > 0 {
            self.current_index -= 1;
            Some(self.states[self.current_index].clone())
        } else {
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
        self.current_index > 0
    }

    /// 是否可以重做
    pub fn can_redo(&self) -> bool {
        self.current_index < self.states.len() - 1
    }
}

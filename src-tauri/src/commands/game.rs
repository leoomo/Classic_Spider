use crate::game::history::History;
use crate::game::state::GameState;
use std::sync::Mutex;
use tauri::State;

/// 游戏管理器
pub struct GameManager {
    pub state: Mutex<GameState>,
    pub history: Mutex<History>,
}

impl Default for GameManager {
    fn default() -> Self {
        let (state, history) = GameState::new(1);

        GameManager {
            state: Mutex::new(state),
            history: Mutex::new(history),
        }
    }
}

/// 创建新游戏
#[tauri::command]
pub fn new_game(difficulty: u8, manager: State<GameManager>) -> GameState {
    let (state, history) = GameState::new(difficulty);

    let mut game_state = manager.state.lock().unwrap();
    *game_state = state.clone();

    let mut game_history = manager.history.lock().unwrap();
    *game_history = history;

    state
}

/// 获取当前游戏状态
#[tauri::command]
pub fn get_state(manager: State<GameManager>) -> GameState {
    manager.state.lock().unwrap().clone()
}

/// 移动卡牌
#[tauri::command]
pub fn move_cards(
    from_col: usize,
    start_idx: usize,
    to_col: usize,
    manager: State<GameManager>,
) -> Result<GameState, String> {
    // 先保存历史
    {
        let mut history = manager.history.lock().unwrap();
        let state = manager.state.lock().unwrap();
        history.push(state.clone());
    }

    // 执行移动
    let mut game = manager.state.lock().unwrap();
    game.move_cards(from_col, start_idx, to_col)?;
    Ok(game.clone())
}

/// 发牌
#[tauri::command]
pub fn deal_cards(manager: State<GameManager>) -> Result<GameState, String> {
    // 先保存历史
    {
        let mut history = manager.history.lock().unwrap();
        let state = manager.state.lock().unwrap();
        history.push(state.clone());
    }
    // 执行发牌
    let mut game = manager.state.lock().unwrap();
    game.deal()?;
    Ok(game.clone())
}

/// 获取提示
#[tauri::command]
pub fn get_hint(manager: State<GameManager>) -> Option<(usize, usize, usize)> {
    manager.state.lock().unwrap().get_hint()
}

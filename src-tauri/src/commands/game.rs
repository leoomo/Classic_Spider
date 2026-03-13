use crate::game::history::History;
use crate::game::state::GameState;
use crate::storage;
use std::sync::Mutex;
use tauri::{AppHandle, State};

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
pub fn new_game(difficulty: u8, manager: State<GameManager>, app_handle: AppHandle) -> GameState {
    let (state, history) = GameState::new(difficulty);

    {
        let mut game_state = manager.state.lock().unwrap();
        *game_state = state.clone();
    }

    {
        let mut game_history = manager.history.lock().unwrap();
        *game_history = history;
    }

    // 删除旧存档（开始新游戏）
    let _ = storage::delete_save(&app_handle);

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
    app_handle: AppHandle,
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

    // 💾 自动存档（每次移动后，异步保存不阻塞 UI）
    let state_to_save = game.clone();
    let app_handle_clone = app_handle.clone();
    std::thread::spawn(move || {
        let _ = storage::save_state(&state_to_save, &app_handle_clone);
    });

    Ok(game.clone())
}

/// 发牌
#[tauri::command]
pub fn deal_cards(manager: State<GameManager>, app_handle: AppHandle) -> Result<GameState, String> {
    // 先保存历史
    {
        let mut history = manager.history.lock().unwrap();
        let state = manager.state.lock().unwrap();
        history.push(state.clone());
    }

    // 执行发牌
    let mut game = manager.state.lock().unwrap();
    game.deal()?;

    // 💾 自动存档（每次发牌后，异步保存不阻塞 UI）
    let state_to_save = game.clone();
    let app_handle_clone = app_handle.clone();
    std::thread::spawn(move || {
        let _ = storage::save_state(&state_to_save, &app_handle_clone);
    });

    Ok(game.clone())
}

/// 获取提示
#[tauri::command]
pub fn get_hint(manager: State<GameManager>) -> Option<(usize, usize, usize)> {
    manager.state.lock().unwrap().get_hint()
}

// ============== 孒档功能 ==============

/// 保存游戏（手动存档）
#[tauri::command]
pub fn save_game(manager: State<GameManager>, app_handle: AppHandle) -> Result<(), String> {
    let state = manager.state.lock().unwrap().clone();
    storage::save_state(&state, &app_handle)
}

/// 加载游戏
#[tauri::command]
pub fn load_game(manager: State<GameManager>, app_handle: AppHandle) -> Result<GameState, String> {
    let state = storage::load_state(&app_handle)?.ok_or("No saved game found".to_string())?;

    // 更新内存中的状态
    {
        let mut game_state = manager.state.lock().unwrap();
        *game_state = state.clone();
    }

    // 重置历史（加载后从新开始记录历史）
    {
        let mut history = manager.history.lock().unwrap();
        *history = History::new(state.clone());
    }

    Ok(state)
}

/// 检查是否有存档
#[tauri::command]
pub fn has_saved_game(app_handle: AppHandle) -> bool {
    storage::has_save(&app_handle)
}

/// 删除存档（新游戏时调用）
#[tauri::command]
pub fn delete_saved_game(app_handle: AppHandle) -> Result<(), String> {
    storage::delete_save(&app_handle)
}

// ============== 撤销/重做 ==============

/// 撤销上一步操作
#[tauri::command]
pub fn undo(manager: State<GameManager>) -> Result<GameState, String> {
    let mut history = manager.history.lock().unwrap();

    if let Some(state) = history.undo() {
        // 更新当前状态
        let mut game_state = manager.state.lock().unwrap();
        *game_state = state.clone();
        Ok(state)
    } else {
        Err("无法撤销".to_string())
    }
}

/// 重做上一步操作
#[tauri::command]
pub fn redo(manager: State<GameManager>) -> Result<GameState, String> {
    let mut history = manager.history.lock().unwrap();

    if let Some(state) = history.redo() {
        // 更新当前状态
        let mut game_state = manager.state.lock().unwrap();
        *game_state = state.clone();
        Ok(state)
    } else {
        Err("无法重做".to_string())
    }
}

/// 检查是否可以撤销
#[tauri::command]
pub fn can_undo(manager: State<GameManager>) -> bool {
    manager.history.lock().unwrap().can_undo()
}

/// 检查是否可以重做
#[tauri::command]
pub fn can_redo(manager: State<GameManager>) -> bool {
    manager.history.lock().unwrap().can_redo()
}

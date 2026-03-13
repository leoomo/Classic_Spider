use std::fs;
use std::path::PathBuf;

use crate::game::state::GameState;
use tauri::Manager;

const SAVE_FILE: &str = "save.json";

/// 获取存档路径
fn get_save_path(app_handle: &tauri::AppHandle) -> PathBuf {
    // 使用 Tauri 2.x 的 path API
    app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory")
        .join(SAVE_FILE)
}

/// 保存游戏状态
pub fn save_state(state: &GameState, app_handle: &tauri::AppHandle) -> Result<(), String> {
    let path = get_save_path(app_handle);
    let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

/// 加载游戏状态
pub fn load_state(app_handle: &tauri::AppHandle) -> Result<Option<GameState>, String> {
    let path = get_save_path(app_handle);
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let state: GameState = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    Ok(Some(state))
}

/// 删除存档
pub fn delete_save(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let path = get_save_path(app_handle);
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("IO error: {}", e))?;
    }
    Ok(())
}

/// 检查是否有存档
pub fn has_save(app_handle: &tauri::AppHandle) -> bool {
    get_save_path(app_handle).exists()
}

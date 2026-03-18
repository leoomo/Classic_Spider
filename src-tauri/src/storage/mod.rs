use std::fs;
use std::path::PathBuf;

use crate::game::state::GameState;
use crate::game::stats::GameStats;
use serde::{de::DeserializeOwned, Serialize};
use tauri::Manager;

const SAVE_FILE: &str = "save.json";
const STATS_FILE: &str = "stats.json";

/// 获取应用数据目录中的文件路径
fn get_path(app_handle: &tauri::AppHandle, filename: &str) -> PathBuf {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");
    // 确保目录存在
    std::fs::create_dir_all(&app_dir).ok();
    app_dir.join(filename)
}

/// 保存数据到 JSON 文件
fn save_json<T: Serialize>(data: &T, path: &PathBuf) -> Result<(), String> {
    let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

/// 从 JSON 文件加载数据
fn load_json<T: DeserializeOwned>(path: &PathBuf) -> Result<Option<T>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let result: T = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    Ok(Some(result))
}

/// 保存游戏状态
pub fn save_state(state: &GameState, app_handle: &tauri::AppHandle) -> Result<(), String> {
    save_json(state, &get_path(app_handle, SAVE_FILE))
}

/// 加载游戏状态
pub fn load_state(app_handle: &tauri::AppHandle) -> Result<Option<GameState>, String> {
    load_json(&get_path(app_handle, SAVE_FILE))
}

/// 删除存档
pub fn delete_save(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let path = get_path(app_handle, SAVE_FILE);
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("IO error: {}", e))?;
    }
    Ok(())
}

/// 检查是否有存档
pub fn has_save(app_handle: &tauri::AppHandle) -> bool {
    get_path(app_handle, SAVE_FILE).exists()
}

/// 保存游戏统计
pub fn save_stats(stats: &GameStats, app_handle: &tauri::AppHandle) -> Result<(), String> {
    // 保存到应用目录
    save_json(stats, &get_path(app_handle, STATS_FILE))?;
    // 同时保存到项目目录用于调试
    let debug_path = std::path::Path::new("logs").join("stats_debug.json");
    let _ = save_json(stats, &debug_path);
    println!("[STORAGE] Stats saved to: {:?}", debug_path);
    Ok(())
}

/// 加载游戏统计
pub fn load_stats(app_handle: &tauri::AppHandle) -> Result<GameStats, String> {
    let path = get_path(app_handle, STATS_FILE);
    match load_json::<GameStats>(&path)? {
        Some(stats) => {
            println!("[STORAGE] Loaded stats from app dir: games_played={:?}", stats.games_played);
            // 复制到调试目录
            let debug_path = std::path::Path::new("logs").join("stats_debug.json");
            let _ = save_json(&stats, &debug_path);
            Ok(stats)
        }
        None => {
            println!("[STORAGE] No stats file found, creating new");
            Ok(GameStats::new())
        }
    }
}

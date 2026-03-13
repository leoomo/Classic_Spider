mod commands;
mod game;

use commands::game::GameManager;
use tauri_plugin_opener::init;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(GameManager::default())
        .invoke_handler(tauri::generate_handler![
            commands::game::new_game,
            commands::game::get_state,
            commands::game::move_cards,
            commands::game::deal_cards,
            commands::game::get_hint
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

mod commands;
mod game;
mod storage;

use commands::game::GameManager;

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
            commands::game::get_hint,
            commands::game::save_game,
            commands::game::load_game,
            commands::game::has_saved_game,
            commands::game::delete_saved_game,
            commands::game::undo,
            commands::game::redo,
            commands::game::can_undo,
            commands::game::can_redo,
            commands::game::debug_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

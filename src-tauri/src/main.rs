// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;

// Import from the lib crate
use app_lib::commands::{self, AppState};
use app_lib::config;
use app_lib::deps;
use app_lib::notifications;
use app_lib::tray;

fn main() {
    let app_config = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Config error: {}", e);
            config::AppConfig::default()
        }
    };

    let hotkeys = app_config.hotkeys.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState {
            config: app_config,
            active_card: Arc::new(Mutex::new(None)),
            recording_handle: Arc::new(Mutex::new(None)),
            recording_path: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            commands::take_screenshot,
            commands::start_recording,
            commands::stop_recording,
            commands::convert_to_gif,
            commands::upload_to_r2,
            commands::post_to_jira,
            commands::upload_and_post,
            commands::search_jira_issues,
            commands::get_active_card,
            commands::set_active_card,
            commands::get_presets,
        ])
        .setup(|app| {
            // Check deps and notify
            let missing = deps::missing_deps();
            if !missing.is_empty() {
                notifications::notify_missing_deps(app.handle(), &missing);
            }

            // Create tray
            tray::create_tray(app.handle())?;

            // Register global shortcuts
            use tauri_plugin_global_shortcut::GlobalShortcutExt;

            let h = hotkeys;
            let shortcuts = [
                (h.screenshot_full.clone(), "screenshot_full"),
                (h.screenshot_region.clone(), "screenshot_region"),
                (h.record_screen.clone(), "record_full"),
                (h.record_region.clone(), "record_region"),
                (h.stop_recording.clone(), "stop_recording"),
            ];

            for (key, action) in shortcuts {
                let action = action.to_string();
                if let Err(e) = app.global_shortcut().on_shortcut(key.as_str(), move |_app, _shortcut, _event| {
                    let _ = _app.emit("tray-action", action.clone());
                }) {
                    eprintln!("Failed to register shortcut '{}': {}", key, e);
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

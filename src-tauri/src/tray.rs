use tauri::{
    AppHandle, Emitter, Manager, image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIcon, TrayIconBuilder},
};
use std::path::PathBuf;

use crate::capture::{self, CaptureMode};
use crate::commands::{AppState, CaptureResult, PendingAction};
use crate::config;

fn install_tray_icons() {
    let icon_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".local/share/icons/hicolor/32x32/apps");
    let _ = std::fs::create_dir_all(&icon_dir);
    let normal = icon_dir.join("jira-proofs.png");
    let recording = icon_dir.join("jira-proofs-recording.png");
    if !normal.exists() {
        let _ = std::fs::write(&normal, include_bytes!("../icons/tray-icon.png"));
    }
    if !recording.exists() {
        let _ = std::fs::write(&recording, include_bytes!("../icons/tray-icon-recording.png"));
    }
}

fn load_icon_by_name(name: &str) -> Option<Image<'static>> {
    let path = dirs::home_dir()?
        .join(".local/share/icons/hicolor/32x32/apps")
        .join(format!("{}.png", name));
    if path.exists() {
        Image::from_path(&path).ok()
    } else {
        None
    }
}

pub fn create_tray(app: &AppHandle) -> Result<TrayIcon, tauri::Error> {
    install_tray_icons();

    let screenshot_full = MenuItem::with_id(app, "screenshot_full", "Screenshot (Full)", true, None::<&str>)?;
    let screenshot_region = MenuItem::with_id(app, "screenshot_region", "Screenshot (Region)", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let record_full = MenuItem::with_id(app, "record_full", "Record Screen", true, None::<&str>)?;
    let record_region = MenuItem::with_id(app, "record_region", "Record Region", true, None::<&str>)?;
    let stop_recording = MenuItem::with_id(app, "stop_recording", "Stop Recording", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let set_card = MenuItem::with_id(app, "set_card", "Set Active Jira Card", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &screenshot_full,
            &screenshot_region,
            &sep1,
            &record_full,
            &record_region,
            &stop_recording,
            &sep2,
            &set_card,
            &settings,
            &sep3,
            &quit,
        ],
    )?;

    let default_icon = load_icon_by_name("jira-proofs")
        .unwrap_or_else(|| Image::from_bytes(include_bytes!("../icons/tray-icon.png")).unwrap());

    let tray = TrayIconBuilder::with_id("main")
        .icon(default_icon)
        .tooltip("Jira Proofs")
        .menu(&menu)
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            match id {
                "quit" => {
                    app.exit(0);
                }
                other => {
                    handle_tray_action(app, other);
                }
            }
        })
        .build(app)?;

    Ok(tray)
}

pub fn update_tooltip(tray: &TrayIcon, card: Option<&str>) {
    let tooltip = match card {
        Some(key) => format!("Jira Proofs — {}", key),
        None => "Jira Proofs".to_string(),
    };
    let _ = tray.set_tooltip(Some(&tooltip));
}

pub fn set_recording_icon(tray: &TrayIcon, recording: bool) {
    let name = if recording { "jira-proofs-recording" } else { "jira-proofs" };
    let icon = load_icon_by_name(name)
        .or_else(|| {
            let bytes: &[u8] = if recording {
                include_bytes!("../icons/tray-icon-recording.png")
            } else {
                include_bytes!("../icons/tray-icon.png")
            };
            Image::from_bytes(bytes).ok()
        });
    if let Some(img) = icon {
        let _ = tray.set_icon(Some(img));
    }
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

async fn set_pending_and_show(app: &AppHandle, action: PendingAction) {
    let state = app.state::<AppState>();
    *state.pending_result.lock().await = Some(action);
    show_main_window(app);
}

pub fn handle_tray_action(app: &AppHandle, action: &str) {
    let app = app.clone();
    let action = action.to_string();
    tauri::async_runtime::spawn(async move {
        // Delay to let tray menu close and release pointer grab
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        match action.as_str() {
            "screenshot_full" | "screenshot_region" => {
                let mode = if action == "screenshot_full" {
                    CaptureMode::FullScreen
                } else {
                    CaptureMode::Region
                };
                let state = app.state::<AppState>();
                let local_dir = config::expand_path(&state.config.storage.local_dir);
                match capture::take_screenshot(mode, &local_dir).await {
                    Ok(path) => {
                        let filename = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();
                        let result = CaptureResult {
                            file_path: path.to_string_lossy().to_string(),
                            filename,
                            is_image: true,
                        };
                        set_pending_and_show(&app, PendingAction::Capture(result)).await;
                    }
                    Err(e) => {
                        eprintln!("Screenshot failed: {}", e);
                    }
                }
            }
            "record_full" | "record_region" => {
                let mode = if action == "record_full" {
                    CaptureMode::FullScreen
                } else {
                    CaptureMode::Region
                };
                let state = app.state::<AppState>();
                if state.recording_handle.lock().await.is_some() {
                    eprintln!("Recording already in progress");
                    return;
                }
                let local_dir = config::expand_path(&state.config.storage.local_dir);
                let _ = std::fs::create_dir_all(&local_dir);

                let region = if mode == CaptureMode::Region {
                    match capture::select_region().await {
                        Ok(r) => Some(r),
                        Err(e) => { eprintln!("Region select failed: {}", e); return; }
                    }
                } else {
                    None
                };

                let filename = capture::generate_filename("recording", "mp4");
                let output_path = local_dir.join(&filename);
                let output_str = output_path.to_string_lossy().to_string();

                let (cmd, args) = capture::build_record_command(mode, &output_str, region);
                match tokio::process::Command::new(&cmd).args(&args).spawn() {
                    Ok(child) => {
                        *state.recording_handle.lock().await = Some(child);
                        *state.recording_path.lock().await = Some(output_str);
                        if let Some(tray) = app.tray_by_id("main") {
                            set_recording_icon(&tray, true);
                        }
                    }
                    Err(e) => eprintln!("Failed to start ffmpeg: {}", e),
                }
            }
            "stop_recording" => {
                let state = app.state::<AppState>();
                let output_path = {
                    let rpath = state.recording_path.lock().await;
                    match rpath.clone() {
                        Some(p) => p,
                        None => { eprintln!("No active recording"); return; }
                    }
                };
                {
                    let mut handle = state.recording_handle.lock().await;
                    if let Some(child) = handle.as_ref() {
                        if let Some(pid) = child.id() {
                            unsafe { libc::kill(pid as libc::pid_t, libc::SIGTERM); }
                        }
                    }
                    if let Some(mut child) = handle.take() {
                        let _ = child.wait().await;
                    }
                }
                *state.recording_path.lock().await = None;
                if let Some(tray) = app.tray_by_id("main") {
                    set_recording_icon(&tray, false);
                }
                let path = std::path::Path::new(&output_path);
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                let is_image = matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif");
                let result = CaptureResult { file_path: output_path, filename, is_image };
                set_pending_and_show(&app, PendingAction::Capture(result)).await;
            }
            "set_card" => {
                set_pending_and_show(&app, PendingAction::SetCard).await;
            }
            "settings" => {
                set_pending_and_show(&app, PendingAction::Settings).await;
            }
            _ => {}
        }
    });
}

use tauri::{
    AppHandle, Emitter,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIcon, TrayIconBuilder},
};

pub fn create_tray(app: &AppHandle) -> Result<TrayIcon, tauri::Error> {
    let screenshot_full = MenuItem::with_id(app, "screenshot_full", "Screenshot (Full)", true, None::<&str>)?;
    let screenshot_region = MenuItem::with_id(app, "screenshot_region", "Screenshot (Region)", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let record_full = MenuItem::with_id(app, "record_full", "Record Screen", true, None::<&str>)?;
    let record_region = MenuItem::with_id(app, "record_region", "Record Region", true, None::<&str>)?;
    let stop_recording = MenuItem::with_id(app, "stop_recording", "Stop Recording", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let set_card = MenuItem::with_id(app, "set_card", "Set Active Jira Card", true, None::<&str>)?;
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
            &sep3,
            &quit,
        ],
    )?;

    let tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Jira Proofs")
        .menu(&menu)
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            match id {
                "quit" => {
                    app.exit(0);
                }
                other => {
                    let _ = app.emit("tray-action", other);
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
    // For now, just change the tooltip to indicate recording state.
    // Icon swapping will be added in Task 14 when icon assets are created.
    if recording {
        let _ = tray.set_tooltip(Some("Jira Proofs — RECORDING"));
    }
}

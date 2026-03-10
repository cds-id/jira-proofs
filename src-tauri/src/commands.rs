use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

use crate::capture::{self, CaptureMode};
use crate::config::{self, AppConfig};
use crate::jira::{self, JiraIssue};
use crate::r2;
use crate::tray;

/// Shared app state
pub struct AppState {
    pub config: AppConfig,
    pub active_card: Arc<Mutex<Option<JiraIssue>>>,
    pub recording_handle: Arc<Mutex<Option<tokio::process::Child>>>,
    pub recording_path: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureResult {
    pub file_path: String,
    pub filename: String,
    pub is_image: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    pub url: String,
}

fn parse_mode(mode: &str) -> CaptureMode {
    match mode {
        "region" => CaptureMode::Region,
        _ => CaptureMode::FullScreen,
    }
}

#[tauri::command]
pub async fn take_screenshot(
    state: State<'_, AppState>,
    mode: String,
) -> Result<CaptureResult, String> {
    let local_dir = config::expand_path(&state.config.storage.local_dir);
    let capture_mode = parse_mode(&mode);
    let path = capture::take_screenshot(capture_mode, &local_dir).await?;
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    Ok(CaptureResult {
        file_path: path.to_string_lossy().to_string(),
        filename,
        is_image: true,
    })
}

#[tauri::command]
pub async fn start_recording(
    app: AppHandle,
    state: State<'_, AppState>,
    mode: String,
) -> Result<String, String> {
    if state.recording_handle.lock().await.is_some() {
        return Err("Recording already in progress".into());
    }
    let local_dir = config::expand_path(&state.config.storage.local_dir);
    std::fs::create_dir_all(&local_dir)
        .map_err(|e| format!("Failed to create dir: {}", e))?;

    let capture_mode = parse_mode(&mode);

    let region = if capture_mode == CaptureMode::Region {
        Some(capture::select_region().await?)
    } else {
        None
    };

    let filename = capture::generate_filename("recording", "mp4");
    let output_path = local_dir.join(&filename);
    let output_str = output_path.to_string_lossy().to_string();

    let (cmd, args) = capture::build_record_command(capture_mode, &output_str, region);
    let child = tokio::process::Command::new(&cmd)
        .args(&args)
        .spawn()
        .map_err(|e| format!("Failed to start ffmpeg: {}", e))?;

    {
        let mut handle = state.recording_handle.lock().await;
        *handle = Some(child);
    }
    {
        let mut rpath = state.recording_path.lock().await;
        *rpath = Some(output_str.clone());
    }

    if let Some(tray) = app.tray_by_id("main") {
        tray::set_recording_icon(&tray, true);
    }

    Ok(output_str)
}

#[tauri::command]
pub async fn stop_recording(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CaptureResult, String> {
    let output_path = {
        let rpath = state.recording_path.lock().await;
        rpath
            .clone()
            .ok_or_else(|| "No active recording".to_string())?
    };

    // Send SIGTERM to the ffmpeg process
    {
        let mut handle = state.recording_handle.lock().await;
        if let Some(child) = handle.as_ref() {
            if let Some(pid) = child.id() {
                unsafe {
                    libc::kill(pid as libc::pid_t, libc::SIGTERM);
                }
            }
        }
        // Wait for the process to finish
        if let Some(mut child) = handle.take() {
            let _ = child.wait().await;
        }
    }

    {
        let mut rpath = state.recording_path.lock().await;
        *rpath = None;
    }

    if let Some(tray) = app.tray_by_id("main") {
        tray::set_recording_icon(&tray, false);
    }

    let path = Path::new(&output_path);
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let is_image = matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif");

    Ok(CaptureResult {
        file_path: output_path,
        filename,
        is_image,
    })
}

#[tauri::command]
pub async fn convert_to_gif(input_path: String) -> Result<CaptureResult, String> {
    let input = Path::new(&input_path);
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("recording");
    let parent = input.parent().unwrap_or(Path::new("."));
    let output_filename = format!("{}.gif", stem);
    let output_path = parent.join(&output_filename);
    let output_str = output_path.to_string_lossy().to_string();

    let (cmd, args) = capture::build_gif_convert_command(&input_path, &output_str);
    let output = tokio::process::Command::new(&cmd)
        .args(&args)
        .output()
        .await
        .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "GIF conversion failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(CaptureResult {
        file_path: output_str,
        filename: output_filename,
        is_image: true,
    })
}

#[tauri::command]
pub async fn upload_to_r2(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<UploadResult, String> {
    let path = Path::new(&file_path);
    let url = r2::upload_file(&state.config.r2, path).await?;
    Ok(UploadResult { url })
}

#[tauri::command]
pub async fn post_to_jira(
    state: State<'_, AppState>,
    issue_key: String,
    preset_title: String,
    description: String,
    url: String,
    is_image: bool,
) -> Result<(), String> {
    jira::post_comment(
        &state.config.jira,
        &issue_key,
        &preset_title,
        &description,
        &url,
        is_image,
    )
    .await
}

#[tauri::command]
pub async fn upload_and_post(
    app: AppHandle,
    state: State<'_, AppState>,
    file_path: String,
    issue_key: String,
    preset_title: String,
    description: String,
    is_image: bool,
) -> Result<String, String> {
    use crate::notifications;

    // Upload to R2
    let path = Path::new(&file_path);
    let url = r2::upload_file(&state.config.r2, path)
        .await
        .map_err(|e| {
            notifications::notify_error(&app, "Upload Failed", &e);
            e
        })?;

    // Post to Jira
    jira::post_comment(
        &state.config.jira,
        &issue_key,
        &preset_title,
        &description,
        &url,
        is_image,
    )
    .await
    .map_err(|e| {
        notifications::notify_error(&app, "Jira Post Failed", &e);
        e
    })?;

    notifications::notify_success(
        &app,
        "Jira Proofs",
        &format!("Posted to {} successfully", issue_key),
    );

    Ok(url)
}

#[tauri::command]
pub async fn search_jira_issues(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<JiraIssue>, String> {
    jira::search_issues(&state.config.jira, &query).await
}

#[tauri::command]
pub async fn get_active_card(
    state: State<'_, AppState>,
) -> Result<Option<JiraIssue>, String> {
    let card = state.active_card.lock().await;
    Ok(card.clone())
}

#[tauri::command]
pub async fn set_active_card(
    app: AppHandle,
    state: State<'_, AppState>,
    card: Option<JiraIssue>,
) -> Result<(), String> {
    let key_ref = card.as_ref().map(|c| c.key.as_str());
    if let Some(tray) = app.tray_by_id("main") {
        tray::update_tooltip(&tray, key_ref);
    }
    let mut active = state.active_card.lock().await;
    *active = card;
    Ok(())
}

#[tauri::command]
pub async fn get_presets(state: State<'_, AppState>) -> Result<Vec<(String, String)>, String> {
    let presets = &state.config.presets;
    Ok(vec![
        ("Bug Evidence".to_string(), presets.bug_evidence.clone()),
        ("Work Evidence".to_string(), presets.work_evidence.clone()),
    ])
}

#[tauri::command]
pub async fn get_hotkeys(state: State<'_, AppState>) -> Result<Vec<(String, String)>, String> {
    let h = &state.config.hotkeys;
    Ok(vec![
        ("Screenshot (Full)".into(), h.screenshot_full.clone()),
        ("Screenshot (Region)".into(), h.screenshot_region.clone()),
        ("Record Screen".into(), h.record_screen.clone()),
        ("Record Region".into(), h.record_region.clone()),
        ("Stop Recording".into(), h.stop_recording.clone()),
    ])
}

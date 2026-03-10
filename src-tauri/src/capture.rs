use chrono::Local;
use std::path::{Path, PathBuf};
use tokio::process::Command as AsyncCommand;

#[derive(Debug, Clone, PartialEq)]
pub enum CaptureMode {
    FullScreen,
    Region,
}

pub fn generate_filename(capture_type: &str, extension: &str) -> String {
    let now = Local::now();
    format!("{}_{}.{}", now.format("%Y-%m-%d_%H%M%S"), capture_type, extension)
}

pub fn build_screenshot_command(mode: CaptureMode, output_path: &str) -> (String, Vec<String>) {
    let mut args = Vec::new();
    if mode == CaptureMode::Region {
        args.push("-s".to_string());
    }
    args.push(output_path.to_string());
    ("scrot".to_string(), args)
}

pub async fn take_screenshot(mode: CaptureMode, local_dir: &Path) -> Result<PathBuf, String> {
    let filename = generate_filename(
        match mode {
            CaptureMode::FullScreen => "screenshot_full",
            CaptureMode::Region => "screenshot_region",
        },
        "png",
    );
    let output_path = local_dir.join(&filename);
    std::fs::create_dir_all(local_dir).map_err(|e| format!("Failed to create dir: {}", e))?;
    let (cmd, args) = build_screenshot_command(mode, output_path.to_str().unwrap());
    let output = AsyncCommand::new(&cmd)
        .args(&args)
        .output()
        .await
        .map_err(|e| format!("Failed to run scrot: {}", e))?;
    if !output.status.success() {
        return Err(format!("scrot failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    Ok(output_path)
}

#[derive(Debug, Clone)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub fn parse_slop_output(output: &str) -> Result<Region, String> {
    let parts: Vec<&str> = output.trim().split(|c| c == 'x' || c == '+').collect();
    if parts.len() != 4 {
        return Err(format!("Invalid slop output: {}", output));
    }
    Ok(Region {
        width: parts[0].parse().map_err(|_| "Invalid width".to_string())?,
        height: parts[1].parse().map_err(|_| "Invalid height".to_string())?,
        x: parts[2].parse().map_err(|_| "Invalid x".to_string())?,
        y: parts[3].parse().map_err(|_| "Invalid y".to_string())?,
    })
}

pub fn build_slop_command() -> (String, Vec<String>) {
    ("slop".to_string(), vec!["--format".to_string(), "%wx%h+%x+%y".to_string()])
}

pub fn build_record_command(mode: CaptureMode, output_path: &str, region: Option<Region>) -> (String, Vec<String>) {
    let mut args = vec!["-y".to_string(), "-f".to_string(), "x11grab".to_string()];
    match (mode, region) {
        (CaptureMode::Region, Some(r)) => {
            args.extend([
                "-video_size".to_string(),
                format!("{}x{}", r.width, r.height),
                "-i".to_string(),
                format!(":0.0+{},{}", r.x, r.y),
            ]);
        }
        _ => {
            args.extend(["-i".to_string(), ":0.0".to_string()]);
        }
    }
    args.extend([
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "ultrafast".to_string(),
        output_path.to_string(),
    ]);
    ("ffmpeg".to_string(), args)
}

pub fn build_gif_convert_command(input_path: &str, output_path: &str) -> (String, Vec<String>) {
    (
        "ffmpeg".to_string(),
        vec![
            "-y".to_string(),
            "-i".to_string(),
            input_path.to_string(),
            "-vf".to_string(),
            "fps=15,scale=640:-1".to_string(),
            output_path.to_string(),
        ],
    )
}

pub async fn select_region() -> Result<Region, String> {
    let (cmd, args) = build_slop_command();
    let output = AsyncCommand::new(&cmd)
        .args(&args)
        .output()
        .await
        .map_err(|e| format!("Failed to run slop: {}", e))?;
    if !output.status.success() {
        return Err("Region selection cancelled".to_string());
    }
    parse_slop_output(&String::from_utf8_lossy(&output.stdout))
}

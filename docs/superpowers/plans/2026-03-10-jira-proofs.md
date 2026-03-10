# Jira Proofs Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Tauri v2 system tray app that captures screenshots/recordings, uploads to R2, and posts evidence comments to Jira Cloud.

**Architecture:** Tauri v2 with Rust backend handling capture (scrot/ffmpeg/slop), R2 uploads (aws-sdk-s3), and Jira REST API. Svelte + TypeScript frontend for the preview popup. Config via TOML file. Global hotkeys for quick capture.

**Tech Stack:** Tauri v2, Rust, Svelte, TypeScript, aws-sdk-s3, reqwest, serde, toml, scrot, ffmpeg, slop

**Spec:** `docs/superpowers/specs/2026-03-10-jira-proofs-design.md`

---

## Chunk 1: Project Scaffolding & Config

### Task 1: Scaffold Tauri v2 + Svelte Project

**Files:**
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/icons/` (tray icons)
- Create: `src/App.svelte`
- Create: `src/main.ts`
- Create: `src/styles.css`
- Create: `package.json`
- Create: `svelte.config.js`
- Create: `vite.config.ts`
- Create: `tsconfig.json`

- [ ] **Step 1: Install Tauri CLI and create project**

```bash
npm create tauri-app@latest . -- --template svelte-ts --manager npm
```

If the directory is not empty, init manually:

```bash
npm init -y
npm install -D @sveltejs/vite-plugin-svelte svelte vite typescript @tauri-apps/cli@latest
npm install @tauri-apps/api@latest
npx tauri init
```

- [ ] **Step 2: Add Rust dependencies to Cargo.toml**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
tauri-plugin-notification = "2"
tauri-plugin-global-shortcut = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
aws-sdk-s3 = "1"
aws-config = "1"
aws-credential-types = "1"
chrono = "0.4"
dirs = "5"
base64 = "0.22"
```

- [ ] **Step 3: Configure tauri.conf.json for tray app**

Write `src-tauri/tauri.conf.json`:

```json
{
  "productName": "jira-proofs",
  "version": "0.1.0",
  "identifier": "com.jiraproofs.app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "label": "main",
        "title": "Jira Proofs",
        "visible": false,
        "width": 500,
        "height": 600,
        "resizable": true,
        "decorations": false,
        "transparent": true,
        "alwaysOnTop": true
      }
    ]
  },
  "bundle": {
    "active": true,
    "icon": ["icons/icon.png"]
  },
  "plugins": {
    "shell": { "open": true },
    "notification": { "all": true },
    "global-shortcut": { "all": true }
  }
}
```

The window starts hidden. The frontend shows it via `appWindow.show()` when a capture event fires, and hides it on close.

- [ ] **Step 4: Verify project builds**

```bash
npm install
cd src-tauri && cargo check && cd ..
npm run tauri dev
```

Expected: App launches with no window (tray-only mode). May show default Tauri tray icon or nothing yet.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri v2 + Svelte project"
```

---

### Task 2: Config Module

**Files:**
- Create: `src-tauri/src/config.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src-tauri/tests/config_test.rs`

- [ ] **Step 1: Write failing test for config parsing**

Create `src-tauri/tests/config_test.rs`:

```rust
use jira_proofs::config::AppConfig;

#[test]
fn test_parse_valid_config() {
    let toml_str = r#"
[jira]
base_url = "https://test.atlassian.net"
email = "test@example.com"
api_token = "token123"
default_project = "TEST"

[r2]
account_id = "acc123"
access_key_id = "key123"
secret_access_key = "secret123"
bucket = "test-bucket"
public_url = "https://assets.example.com"

[hotkeys]
screenshot_full = "Print"
screenshot_region = "Shift+Print"
record_screen = "Ctrl+Alt+R"
record_region = "Ctrl+Alt+Shift+R"
stop_recording = "Ctrl+Alt+S"

[storage]
local_dir = "~/Pictures/jira-proofs"

[presets]
bug_evidence = "Bug: {description}\n{url}"
work_evidence = "Work: {description}\n{url}"
"#;
    let config: AppConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.jira.base_url, "https://test.atlassian.net");
    assert_eq!(config.r2.bucket, "test-bucket");
    assert_eq!(config.hotkeys.screenshot_full, "Print");
    assert_eq!(config.hotkeys.stop_recording, "Ctrl+Alt+S");
    assert_eq!(config.storage.local_dir, "~/Pictures/jira-proofs");
}

#[test]
fn test_config_default_creation() {
    let config = AppConfig::default();
    assert_eq!(config.hotkeys.screenshot_full, "Print");
    assert_eq!(config.hotkeys.stop_recording, "Ctrl+Alt+S");
}

#[test]
fn test_preset_template_rendering() {
    let template = "Bug: {description}\n{url}";
    let rendered = jira_proofs::config::render_template(
        template,
        "Login broken",
        "https://assets.example.com/img.png",
    );
    assert_eq!(rendered, "Bug: Login broken\nhttps://assets.example.com/img.png");
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd src-tauri && cargo test --test config_test 2>&1
```

Expected: FAIL — module `config` not found.

- [ ] **Step 3: Implement config module**

Create `src-tauri/src/config.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub jira: JiraConfig,
    pub r2: R2Config,
    pub hotkeys: HotkeyConfig,
    pub storage: StorageConfig,
    pub presets: PresetsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraConfig {
    pub base_url: String,
    pub email: String,
    pub api_token: String,
    pub default_project: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2Config {
    pub account_id: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub bucket: String,
    pub public_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub screenshot_full: String,
    pub screenshot_region: String,
    pub record_screen: String,
    pub record_region: String,
    pub stop_recording: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub local_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetsConfig {
    pub bug_evidence: String,
    pub work_evidence: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            jira: JiraConfig {
                base_url: "https://yourteam.atlassian.net".into(),
                email: "you@example.com".into(),
                api_token: "your-jira-api-token".into(),
                default_project: "PROJ".into(),
            },
            r2: R2Config {
                account_id: "your-cf-account-id".into(),
                access_key_id: "your-r2-access-key".into(),
                secret_access_key: "your-r2-secret".into(),
                bucket: "jira-proofs".into(),
                public_url: "https://assets.yourdomain.com".into(),
            },
            hotkeys: HotkeyConfig {
                screenshot_full: "Print".into(),
                screenshot_region: "Shift+Print".into(),
                record_screen: "Ctrl+Alt+R".into(),
                record_region: "Ctrl+Alt+Shift+R".into(),
                stop_recording: "Ctrl+Alt+S".into(),
            },
            storage: StorageConfig {
                local_dir: "~/Pictures/jira-proofs".into(),
            },
            presets: PresetsConfig {
                bug_evidence: "🐛 **Bug Evidence**\n\n{description}\n\n![]({url})".into(),
                work_evidence: "✅ **Work Evidence**\n\n{description}\n\n![]({url})".into(),
            },
        }
    }
}

/// Expand ~ to home directory
pub fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

/// Get config file path: ~/.config/jira-proofs/config.toml
pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("jira-proofs")
        .join("config.toml")
}

/// Load config from disk, or create default template if missing
pub fn load_config() -> Result<AppConfig, String> {
    let path = config_path();
    if !path.exists() {
        create_default_config(&path)?;
        return Err(format!(
            "Config created at {}. Please fill in your credentials.",
            path.display()
        ));
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    toml::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))
}

fn create_default_config(path: &PathBuf) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let default = AppConfig::default();
    let content = toml::to_string_pretty(&default)
        .map_err(|e| format!("Failed to serialize default config: {}", e))?;
    fs::write(path, &content)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    // chmod 600
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        fs::set_permissions(path, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }
    Ok(())
}

/// Render a preset template by replacing {description} and {url} placeholders
pub fn render_template(template: &str, description: &str, url: &str) -> String {
    template
        .replace("{description}", description)
        .replace("{url}", url)
}
```

Create `src-tauri/src/lib.rs` (will be appended to in subsequent tasks):

```rust
pub mod config;
```

> **Note:** Each subsequent task appends a `pub mod X;` line to `lib.rs`. The final cumulative state is:
> ```rust
> pub mod capture;
> pub mod commands;
> pub mod config;
> pub mod deps;
> pub mod jira;
> pub mod notifications;
> pub mod r2;
> pub mod tray;
> ```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd src-tauri && cargo test --test config_test -v 2>&1
```

Expected: 3 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/config.rs src-tauri/src/lib.rs src-tauri/tests/config_test.rs
git commit -m "feat: add config module with TOML parsing and template rendering"
```

---

## Chunk 2: Capture Engine & System Dependency Checks

### Task 3: System Dependency Checker

**Files:**
- Create: `src-tauri/src/deps.rs`
- Create: `src-tauri/tests/deps_test.rs`

- [ ] **Step 1: Write failing test**

Create `src-tauri/tests/deps_test.rs`:

```rust
use jira_proofs::deps::{check_dependency, DependencyStatus};

#[test]
fn test_check_existing_binary() {
    // `ls` should exist on any Linux system
    let status = check_dependency("ls");
    assert!(matches!(status, DependencyStatus::Found(_)));
}

#[test]
fn test_check_missing_binary() {
    let status = check_dependency("nonexistent_binary_xyz_123");
    assert!(matches!(status, DependencyStatus::Missing(_)));
}

#[test]
fn test_check_all_dependencies() {
    let results = jira_proofs::deps::check_all();
    assert_eq!(results.len(), 3); // scrot, ffmpeg, slop
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd src-tauri && cargo test --test deps_test 2>&1
```

Expected: FAIL — module `deps` not found.

- [ ] **Step 3: Implement deps module**

Create `src-tauri/src/deps.rs`:

```rust
use std::process::Command;

pub enum DependencyStatus {
    Found(String),
    Missing(String),
}

const REQUIRED_DEPS: &[&str] = &["scrot", "ffmpeg", "slop"];

pub fn check_dependency(name: &str) -> DependencyStatus {
    match Command::new("which").arg(name).output() {
        Ok(output) if output.status.success() => {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            DependencyStatus::Found(path)
        }
        _ => DependencyStatus::Missing(name.to_string()),
    }
}

pub fn check_all() -> Vec<(String, DependencyStatus)> {
    REQUIRED_DEPS
        .iter()
        .map(|dep| (dep.to_string(), check_dependency(dep)))
        .collect()
}

pub fn missing_deps() -> Vec<String> {
    check_all()
        .into_iter()
        .filter_map(|(name, status)| match status {
            DependencyStatus::Missing(_) => Some(name),
            DependencyStatus::Found(_) => None,
        })
        .collect()
}
```

Add to `src-tauri/src/lib.rs`:

```rust
pub mod deps;
```

- [ ] **Step 4: Run tests**

```bash
cd src-tauri && cargo test --test deps_test -v 2>&1
```

Expected: 3 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/deps.rs src-tauri/src/lib.rs src-tauri/tests/deps_test.rs
git commit -m "feat: add system dependency checker for scrot/ffmpeg/slop"
```

---

### Task 4: Capture Engine — Screenshots

**Files:**
- Create: `src-tauri/src/capture.rs`
- Create: `src-tauri/tests/capture_test.rs`

- [ ] **Step 1: Write failing test**

Create `src-tauri/tests/capture_test.rs`:

```rust
use jira_proofs::capture::{build_screenshot_command, CaptureMode, generate_filename};

#[test]
fn test_generate_filename_screenshot() {
    let name = generate_filename("screenshot_full", "png");
    // Format: 2026-03-10_143022_screenshot_full.png
    assert!(name.ends_with("_screenshot_full.png"));
    assert!(name.len() > 20);
}

#[test]
fn test_build_full_screenshot_command() {
    let (cmd, args) = build_screenshot_command(CaptureMode::FullScreen, "/tmp/test.png");
    assert_eq!(cmd, "scrot");
    assert!(args.contains(&"/tmp/test.png".to_string()));
}

#[test]
fn test_build_region_screenshot_command() {
    let (cmd, args) = build_screenshot_command(CaptureMode::Region, "/tmp/test.png");
    assert_eq!(cmd, "scrot");
    assert!(args.contains(&"-s".to_string()));
    assert!(args.contains(&"/tmp/test.png".to_string()));
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd src-tauri && cargo test --test capture_test 2>&1
```

Expected: FAIL — module `capture` not found.

- [ ] **Step 3: Implement capture module (screenshot part)**

Create `src-tauri/src/capture.rs`:

```rust
use chrono::Local;
use std::path::{Path, PathBuf};
use tokio::process::Command as AsyncCommand;

#[derive(Debug, Clone, PartialEq)]
pub enum CaptureMode {
    FullScreen,
    Region,
}

/// Generate a timestamped filename
pub fn generate_filename(capture_type: &str, extension: &str) -> String {
    let now = Local::now();
    format!(
        "{}_{}.{}",
        now.format("%Y-%m-%d_%H%M%S"),
        capture_type,
        extension
    )
}

/// Build the scrot command for screenshots
pub fn build_screenshot_command(mode: CaptureMode, output_path: &str) -> (String, Vec<String>) {
    let mut args = Vec::new();
    if mode == CaptureMode::Region {
        args.push("-s".to_string());
    }
    args.push(output_path.to_string());
    ("scrot".to_string(), args)
}

/// Take a screenshot asynchronously
pub async fn take_screenshot(
    mode: CaptureMode,
    local_dir: &Path,
) -> Result<PathBuf, String> {
    let filename = generate_filename(
        match mode {
            CaptureMode::FullScreen => "screenshot_full",
            CaptureMode::Region => "screenshot_region",
        },
        "png",
    );
    let output_path = local_dir.join(&filename);

    // Ensure local_dir exists
    std::fs::create_dir_all(local_dir)
        .map_err(|e| format!("Failed to create dir: {}", e))?;

    let (cmd, args) = build_screenshot_command(mode, output_path.to_str().unwrap());

    let output = AsyncCommand::new(&cmd)
        .args(&args)
        .output()
        .await
        .map_err(|e| format!("Failed to run scrot: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("scrot failed: {}", stderr));
    }

    Ok(output_path)
}
```

Add to `src-tauri/src/lib.rs`:

```rust
pub mod capture;
```

- [ ] **Step 4: Run tests**

```bash
cd src-tauri && cargo test --test capture_test -v 2>&1
```

Expected: 3 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/capture.rs src-tauri/src/lib.rs src-tauri/tests/capture_test.rs
git commit -m "feat: add capture engine with screenshot support"
```

---

### Task 5: Capture Engine — Screen Recording

**Files:**
- Modify: `src-tauri/src/capture.rs`
- Modify: `src-tauri/tests/capture_test.rs`

- [ ] **Step 1: Write failing tests for recording commands**

Add to `src-tauri/tests/capture_test.rs`:

```rust
use jira_proofs::capture::{
    build_record_command, build_gif_convert_command, build_slop_command, parse_slop_output,
};

#[test]
fn test_parse_slop_output() {
    let region = parse_slop_output("800x600+100+200").unwrap();
    assert_eq!(region.width, 800);
    assert_eq!(region.height, 600);
    assert_eq!(region.x, 100);
    assert_eq!(region.y, 200);
}

#[test]
fn test_parse_slop_output_invalid() {
    assert!(parse_slop_output("invalid").is_err());
}

#[test]
fn test_build_fullscreen_record_command() {
    let (cmd, args) = build_record_command(CaptureMode::FullScreen, "/tmp/test.mp4", None);
    assert_eq!(cmd, "ffmpeg");
    assert!(args.contains(&"-f".to_string()));
    assert!(args.contains(&"x11grab".to_string()));
    assert!(args.contains(&"/tmp/test.mp4".to_string()));
}

#[test]
fn test_build_region_record_command() {
    let region = jira_proofs::capture::Region {
        x: 100,
        y: 200,
        width: 800,
        height: 600,
    };
    let (cmd, args) = build_record_command(CaptureMode::Region, "/tmp/test.mp4", Some(region));
    assert_eq!(cmd, "ffmpeg");
    assert!(args.contains(&"800x600".to_string()));
    // Check offset is in the -i argument: :0.0+100,200
    let i_arg = args.iter().find(|a| a.contains("+100,200"));
    assert!(i_arg.is_some());
}

#[test]
fn test_build_gif_convert_command() {
    let (cmd, args) = build_gif_convert_command("/tmp/test.mp4", "/tmp/test.gif");
    assert_eq!(cmd, "ffmpeg");
    assert!(args.contains(&"/tmp/test.mp4".to_string()));
    assert!(args.contains(&"/tmp/test.gif".to_string()));
}

#[test]
fn test_build_slop_command() {
    let (cmd, args) = build_slop_command();
    assert_eq!(cmd, "slop");
    assert!(args.contains(&"--format".to_string()));
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test --test capture_test 2>&1
```

Expected: FAIL — functions not found.

- [ ] **Step 3: Add recording implementation to capture.rs**

Add to `src-tauri/src/capture.rs`:

```rust
#[derive(Debug, Clone)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Parse slop output format "WxH+X+Y" into Region
pub fn parse_slop_output(output: &str) -> Result<Region, String> {
    // slop outputs: WxH+X+Y
    let parts: Vec<&str> = output.trim().split(|c| c == 'x' || c == '+').collect();
    if parts.len() != 4 {
        return Err(format!("Invalid slop output: {}", output));
    }
    Ok(Region {
        width: parts[0].parse().map_err(|_| "Invalid width")?,
        height: parts[1].parse().map_err(|_| "Invalid height")?,
        x: parts[2].parse().map_err(|_| "Invalid x")?,
        y: parts[3].parse().map_err(|_| "Invalid y")?,
    })
}

/// Build slop command for region selection
pub fn build_slop_command() -> (String, Vec<String>) {
    ("slop".to_string(), vec!["--format".to_string(), "%wx%h+%x+%y".to_string()])
}

/// Build ffmpeg command for screen recording
pub fn build_record_command(
    mode: CaptureMode,
    output_path: &str,
    region: Option<Region>,
) -> (String, Vec<String>) {
    let mut args = vec![
        "-y".to_string(),
        "-f".to_string(),
        "x11grab".to_string(),
    ];

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
            args.extend([
                "-i".to_string(),
                ":0.0".to_string(),
            ]);
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

/// Build ffmpeg command for MP4 → GIF conversion
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

/// Select a screen region interactively using slop
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_slop_output(&stdout)
}
```

- [ ] **Step 4: Run tests**

```bash
cd src-tauri && cargo test --test capture_test -v 2>&1
```

Expected: All tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/capture.rs src-tauri/tests/capture_test.rs
git commit -m "feat: add screen recording and region selection to capture engine"
```

---

## Chunk 3: R2 & Jira Clients

### Task 6: R2 Upload Client

**Files:**
- Create: `src-tauri/src/r2.rs`
- Create: `src-tauri/tests/r2_test.rs`

- [ ] **Step 1: Write failing test for R2 URL construction**

Create `src-tauri/tests/r2_test.rs`:

```rust
use jira_proofs::r2::{build_object_key, build_public_url, content_type_for_extension};

#[test]
fn test_build_object_key() {
    let key = build_object_key("2026-03-10_143022_screenshot_full.png");
    assert!(key.starts_with("captures/"));
    assert!(key.ends_with("/2026-03-10_143022_screenshot_full.png"));
    // Verify structure: captures/{date}/{filename}
    let parts: Vec<&str> = key.split('/').collect();
    assert_eq!(parts.len(), 3);
}

#[test]
fn test_build_public_url() {
    let url = build_public_url(
        "https://assets.example.com",
        "captures/2026-03-10/test.png",
    );
    assert_eq!(url, "https://assets.example.com/captures/2026-03-10/test.png");
}

#[test]
fn test_content_type_png() {
    assert_eq!(content_type_for_extension("png"), "image/png");
}

#[test]
fn test_content_type_mp4() {
    assert_eq!(content_type_for_extension("mp4"), "video/mp4");
}

#[test]
fn test_content_type_gif() {
    assert_eq!(content_type_for_extension("gif"), "image/gif");
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test --test r2_test 2>&1
```

Expected: FAIL — module `r2` not found.

- [ ] **Step 3: Implement R2 module**

Create `src-tauri/src/r2.rs`:

```rust
use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use chrono::Local;
use std::path::Path;

use crate::config::R2Config;

pub fn content_type_for_extension(ext: &str) -> &'static str {
    match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    }
}

pub fn build_object_key(filename: &str) -> String {
    let date = Local::now().format("%Y-%m-%d");
    format!("captures/{}/{}", date, filename)
}

pub fn build_public_url(public_base: &str, object_key: &str) -> String {
    format!("{}/{}", public_base.trim_end_matches('/'), object_key)
}

fn create_s3_client(config: &R2Config) -> Client {
    let credentials = Credentials::new(
        &config.access_key_id,
        &config.secret_access_key,
        None,
        None,
        "jira-proofs",
    );

    let s3_config = S3ConfigBuilder::new()
        .endpoint_url(format!(
            "https://{}.r2.cloudflarestorage.com",
            config.account_id
        ))
        .region(Region::new("auto"))
        .credentials_provider(credentials)
        .force_path_style(true)
        .build();

    Client::from_conf(s3_config)
}

/// Upload a file to R2 and return the public URL
pub async fn upload_file(
    config: &R2Config,
    file_path: &Path,
) -> Result<String, String> {
    let client = create_s3_client(config);

    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid filename")?;

    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");

    let object_key = build_object_key(filename);
    let content_type = content_type_for_extension(extension);

    let body = ByteStream::from_path(file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    client
        .put_object()
        .bucket(&config.bucket)
        .key(&object_key)
        .body(body)
        .content_type(content_type)
        .send()
        .await
        .map_err(|e| format!("R2 upload failed: {}", e))?;

    Ok(build_public_url(&config.public_url, &object_key))
}
```

Add to `src-tauri/src/lib.rs`:

```rust
pub mod r2;
```

- [ ] **Step 4: Run tests**

```bash
cd src-tauri && cargo test --test r2_test -v 2>&1
```

Expected: 5 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/r2.rs src-tauri/src/lib.rs src-tauri/tests/r2_test.rs
git commit -m "feat: add R2 upload client with S3-compatible API"
```

---

### Task 7: Jira Client

**Files:**
- Create: `src-tauri/src/jira.rs`
- Create: `src-tauri/tests/jira_test.rs`

- [ ] **Step 1: Write failing tests for ADF construction and auth**

Create `src-tauri/tests/jira_test.rs`:

```rust
use jira_proofs::jira::{
    build_auth_header, build_adf_comment, build_adf_image_comment, build_adf_link_comment,
};

#[test]
fn test_build_auth_header() {
    let header = build_auth_header("user@example.com", "token123");
    // base64("user@example.com:token123")
    assert!(header.starts_with("Basic "));
    assert!(header.len() > 10);
}

#[test]
fn test_build_adf_comment_with_image() {
    let adf = build_adf_image_comment("Bug found on login page", "https://r2.example.com/img.png");
    let json = serde_json::to_string(&adf).unwrap();
    assert!(json.contains("Bug found on login page"));
    assert!(json.contains("https://r2.example.com/img.png"));
    assert!(json.contains("\"type\":\"doc\""));
}

#[test]
fn test_build_adf_comment_with_link() {
    let adf = build_adf_link_comment("Recording of the bug", "https://r2.example.com/vid.mp4");
    let json = serde_json::to_string(&adf).unwrap();
    assert!(json.contains("Recording of the bug"));
    assert!(json.contains("https://r2.example.com/vid.mp4"));
}

#[test]
fn test_build_adf_comment_renders_preset() {
    let adf = build_adf_comment(
        "Bug Evidence",
        "Login button broken",
        "https://r2.example.com/img.png",
        true, // is_image
    );
    let json = serde_json::to_string(&adf).unwrap();
    assert!(json.contains("Bug Evidence"));
    assert!(json.contains("Login button broken"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test --test jira_test 2>&1
```

Expected: FAIL — module `jira` not found.

- [ ] **Step 3: Implement Jira module**

Create `src-tauri/src/jira.rs`:

```rust
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::JiraConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssue {
    pub key: String,
    pub summary: String,
}

pub fn build_auth_header(email: &str, api_token: &str) -> String {
    let credentials = format!("{}:{}", email, api_token);
    format!("Basic {}", BASE64.encode(credentials.as_bytes()))
}

/// Build an ADF document with a heading, description text, and an inline image
pub fn build_adf_image_comment(description: &str, image_url: &str) -> Value {
    json!({
        "version": 1,
        "type": "doc",
        "content": [
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "text": description
                    }
                ]
            },
            {
                "type": "mediaSingle",
                "attrs": { "layout": "center" },
                "content": [
                    {
                        "type": "media",
                        "attrs": {
                            "type": "external",
                            "url": image_url
                        }
                    }
                ]
            }
        ]
    })
}

/// Build an ADF document with description and a link to a video/GIF
pub fn build_adf_link_comment(description: &str, link_url: &str) -> Value {
    json!({
        "version": 1,
        "type": "doc",
        "content": [
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "text": description
                    }
                ]
            },
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "text": link_url,
                        "marks": [
                            {
                                "type": "link",
                                "attrs": { "href": link_url }
                            }
                        ]
                    }
                ]
            }
        ]
    })
}

/// Build a full ADF comment with preset heading, description, and media
pub fn build_adf_comment(
    preset_title: &str,
    description: &str,
    url: &str,
    is_image: bool,
) -> Value {
    let mut content = vec![
        json!({
            "type": "heading",
            "attrs": { "level": 3 },
            "content": [
                {
                    "type": "text",
                    "text": preset_title
                }
            ]
        }),
        json!({
            "type": "paragraph",
            "content": [
                {
                    "type": "text",
                    "text": description
                }
            ]
        }),
    ];

    if is_image {
        content.push(json!({
            "type": "mediaSingle",
            "attrs": { "layout": "center" },
            "content": [
                {
                    "type": "media",
                    "attrs": {
                        "type": "external",
                        "url": url
                    }
                }
            ]
        }));
    } else {
        content.push(json!({
            "type": "paragraph",
            "content": [
                {
                    "type": "text",
                    "text": url,
                    "marks": [
                        {
                            "type": "link",
                            "attrs": { "href": url }
                        }
                    ]
                }
            ]
        }));
    }

    json!({
        "version": 1,
        "type": "doc",
        "content": content
    })
}

/// Search Jira issues by JQL
pub async fn search_issues(
    config: &JiraConfig,
    query: &str,
) -> Result<Vec<JiraIssue>, String> {
    let client = Client::new();
    let auth = build_auth_header(&config.email, &config.api_token);

    let jql = if query.is_empty() {
        format!(
            "project = {} AND status != Done ORDER BY updated DESC",
            config.default_project
        )
    } else {
        format!(
            "project = {} AND summary ~ \"{}\" AND status != Done ORDER BY updated DESC",
            config.default_project, query
        )
    };

    let url = format!("{}/rest/api/3/search", config.base_url);

    let response = client
        .get(&url)
        .header("Authorization", &auth)
        .header("Accept", "application/json")
        .query(&[("jql", &jql), ("maxResults", &"20".to_string()), ("fields", &"summary".to_string())])
        .send()
        .await
        .map_err(|e| format!("Jira search failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Jira API error {}: {}", status, body));
    }

    let body: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Jira response: {}", e))?;

    let issues = body["issues"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|issue| {
            let key = issue["key"].as_str()?.to_string();
            let summary = issue["fields"]["summary"].as_str()?.to_string();
            Some(JiraIssue { key, summary })
        })
        .collect();

    Ok(issues)
}

/// Post a comment to a Jira issue
pub async fn post_comment(
    config: &JiraConfig,
    issue_key: &str,
    preset_title: &str,
    description: &str,
    url: &str,
    is_image: bool,
) -> Result<(), String> {
    let client = Client::new();
    let auth = build_auth_header(&config.email, &config.api_token);

    let adf = build_adf_comment(preset_title, description, url, is_image);

    let api_url = format!(
        "{}/rest/api/3/issue/{}/comment",
        config.base_url, issue_key
    );

    let response = client
        .post(&api_url)
        .header("Authorization", &auth)
        .header("Content-Type", "application/json")
        .json(&json!({ "body": adf }))
        .send()
        .await
        .map_err(|e| format!("Failed to post comment: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Jira comment failed {}: {}", status, body));
    }

    Ok(())
}
```

Add to `src-tauri/src/lib.rs`:

```rust
pub mod jira;
```

- [ ] **Step 4: Run tests**

```bash
cd src-tauri && cargo test --test jira_test -v 2>&1
```

Expected: 4 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/jira.rs src-tauri/src/lib.rs src-tauri/tests/jira_test.rs
git commit -m "feat: add Jira client with ADF comment builder and issue search"
```

---

## Chunk 4: Tauri IPC Commands & Tray Manager

### Task 8: Tauri IPC Commands

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Implement Tauri commands**

Create `src-tauri/src/commands.rs`:

```rust
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

/// Take a screenshot and return the file path
#[tauri::command]
pub async fn take_screenshot(
    state: State<'_, AppState>,
    mode: String,
) -> Result<CaptureResult, String> {
    let capture_mode = match mode.as_str() {
        "full" => CaptureMode::FullScreen,
        "region" => CaptureMode::Region,
        _ => return Err("Invalid mode: use 'full' or 'region'".into()),
    };

    let local_dir = config::expand_path(&state.config.storage.local_dir);
    let path = capture::take_screenshot(capture_mode, &local_dir).await?;
    let filename = path.file_name().unwrap().to_string_lossy().to_string();

    Ok(CaptureResult {
        file_path: path.to_string_lossy().to_string(),
        filename,
        is_image: true,
    })
}

/// Start screen recording
#[tauri::command]
pub async fn start_recording(
    app: AppHandle,
    state: State<'_, AppState>,
    mode: String,
) -> Result<String, String> {
    // Check if already recording
    let handle = state.recording_handle.lock().await;
    if handle.is_some() {
        return Err("Already recording".into());
    }
    drop(handle);

    let capture_mode = match mode.as_str() {
        "full" => CaptureMode::FullScreen,
        "region" => CaptureMode::Region,
        _ => return Err("Invalid mode".into()),
    };

    let region = if capture_mode == CaptureMode::Region {
        Some(capture::select_region().await?)
    } else {
        None
    };

    let local_dir = config::expand_path(&state.config.storage.local_dir);
    std::fs::create_dir_all(&local_dir)
        .map_err(|e| format!("Failed to create dir: {}", e))?;

    let filename = capture::generate_filename("recording", "mp4");
    let output_path = local_dir.join(&filename);
    let output_str = output_path.to_string_lossy().to_string();

    let (cmd, args) = capture::build_record_command(
        capture_mode,
        &output_str,
        region,
    );

    let child = tokio::process::Command::new(&cmd)
        .args(&args)
        .spawn()
        .map_err(|e| format!("Failed to start recording: {}", e))?;

    *state.recording_handle.lock().await = Some(child);
    *state.recording_path.lock().await = Some(output_str.clone());

    // Update tray icon to recording indicator
    if let Some(tray) = app.tray_by_id("main") {
        tray::set_recording_icon(&tray, true);
    }

    Ok(output_str)
}

/// Stop screen recording and return the file path
#[tauri::command]
pub async fn stop_recording(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CaptureResult, String> {
    let mut handle = state.recording_handle.lock().await;
    let child = handle.take().ok_or("Not recording")?;

    // Send SIGTERM to ffmpeg for clean shutdown
    let pid = child.id().ok_or("No PID")?;
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }

    // Wait for process to finish
    drop(handle);

    // Small delay to let ffmpeg finalize the file
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Reset tray icon
    if let Some(tray) = app.tray_by_id("main") {
        tray::set_recording_icon(&tray, false);
    }

    let path = state.recording_path.lock().await.take()
        .ok_or("No recording path")?;

    let filename = Path::new(&path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    Ok(CaptureResult {
        file_path: path,
        filename,
        is_image: false,
    })
}

/// Convert MP4 to GIF
#[tauri::command]
pub async fn convert_to_gif(
    input_path: String,
) -> Result<CaptureResult, String> {
    let gif_path = input_path.replace(".mp4", ".gif");
    let (cmd, args) = capture::build_gif_convert_command(&input_path, &gif_path);

    let output = tokio::process::Command::new(&cmd)
        .args(&args)
        .output()
        .await
        .map_err(|e| format!("GIF conversion failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("GIF conversion failed: {}", stderr));
    }

    let filename = Path::new(&gif_path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    Ok(CaptureResult {
        file_path: gif_path,
        filename,
        is_image: true, // GIF embeds as image in Jira
    })
}

/// Upload file to R2 and return the public URL
#[tauri::command]
pub async fn upload_to_r2(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<UploadResult, String> {
    let path = Path::new(&file_path);
    let url = r2::upload_file(&state.config.r2, path).await?;
    Ok(UploadResult { url })
}

/// Post a comment to Jira
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

/// Search Jira issues
#[tauri::command]
pub async fn search_jira_issues(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<JiraIssue>, String> {
    jira::search_issues(&state.config.jira, &query).await
}

/// Get/set active Jira card
#[tauri::command]
pub async fn get_active_card(
    state: State<'_, AppState>,
) -> Result<Option<JiraIssue>, String> {
    Ok(state.active_card.lock().await.clone())
}

#[tauri::command]
pub async fn set_active_card(
    app: AppHandle,
    state: State<'_, AppState>,
    card: Option<JiraIssue>,
) -> Result<(), String> {
    let key = card.as_ref().map(|c| c.key.as_str());
    // Update tray tooltip with active card
    if let Some(tray) = app.tray_by_id("main") {
        tray::update_tooltip(&tray, key);
    }
    *state.active_card.lock().await = card;
    Ok(())
}

/// Get preset names and titles from config
#[tauri::command]
pub async fn get_presets(
    state: State<'_, AppState>,
) -> Result<Vec<(String, String)>, String> {
    let mut presets = Vec::new();
    if !state.config.presets.bug_evidence.is_empty() {
        presets.push(("bug_evidence".into(), "Bug Evidence".into()));
    }
    if !state.config.presets.work_evidence.is_empty() {
        presets.push(("work_evidence".into(), "Work Evidence".into()));
    }
    Ok(presets)
}
```

Add to `src-tauri/src/lib.rs`:

```rust
pub mod commands;
```

Also add `libc` to `src-tauri/Cargo.toml`:

```toml
libc = "0.2"
```

- [ ] **Step 2: Verify it compiles**

```bash
cd src-tauri && cargo check 2>&1
```

Expected: Compiles without errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat: add Tauri IPC commands for capture, upload, and Jira"
```

---

### Task 9: Tray Manager & App Entry Point

**Files:**
- Create: `src-tauri/src/tray.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Implement tray module**

Create `src-tauri/src/tray.rs`:

```rust
use tauri::{
    AppHandle, Manager,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    Emitter,
};

pub fn create_tray(app: &AppHandle) -> Result<TrayIcon, tauri::Error> {
    let screenshot_full = MenuItem::with_id(app, "screenshot_full", "Screenshot (Full)", true, None::<&str>)?;
    let screenshot_region = MenuItem::with_id(app, "screenshot_region", "Screenshot (Region)", true, None::<&str>)?;
    let separator1 = PredefinedMenuItem::separator(app)?;
    let record_full = MenuItem::with_id(app, "record_full", "Record Screen", true, None::<&str>)?;
    let record_region = MenuItem::with_id(app, "record_region", "Record Region", true, None::<&str>)?;
    let stop_recording = MenuItem::with_id(app, "stop_recording", "Stop Recording", true, None::<&str>)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let set_card = MenuItem::with_id(app, "set_card", "Set Active Jira Card", true, None::<&str>)?;
    let separator3 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[
        &screenshot_full,
        &screenshot_region,
        &separator1,
        &record_full,
        &record_region,
        &stop_recording,
        &separator2,
        &set_card,
        &separator3,
        &quit,
    ])?;

    let tray = TrayIconBuilder::with_id(app, "main")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Jira Proofs")
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();
            match id {
                "screenshot_full" => {
                    let _ = app.emit("tray-action", "screenshot_full");
                }
                "screenshot_region" => {
                    let _ = app.emit("tray-action", "screenshot_region");
                }
                "record_full" => {
                    let _ = app.emit("tray-action", "record_full");
                }
                "record_region" => {
                    let _ = app.emit("tray-action", "record_region");
                }
                "stop_recording" => {
                    let _ = app.emit("tray-action", "stop_recording");
                }
                "set_card" => {
                    let _ = app.emit("tray-action", "set_card");
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(tray)
}
```

- [ ] **Step 2: Implement main.rs with tray and global shortcuts**

Update `src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

use jira_proofs::commands::AppState;
use jira_proofs::config;
use jira_proofs::deps;
use jira_proofs::notifications;
use jira_proofs::tray;
use jira_proofs::commands;

fn main() {
    // Load config
    let app_config = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Config error: {}", e);
            eprintln!("Please configure ~/.config/jira-proofs/config.toml");
            // Use default config so app can still launch
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
            // Check dependencies and notify if missing
            let missing = deps::missing_deps();
            if !missing.is_empty() {
                notifications::notify_missing_deps(app.handle(), &missing);
            }

            // Create tray icon
            let _ = tray::create_tray(app.handle())?;

            // Register global shortcuts
            use tauri_plugin_global_shortcut::GlobalShortcutExt;

            let h = hotkeys.clone();

            app.global_shortcut().on_shortcut(
                h.screenshot_full.parse().unwrap(),
                move |_app, _shortcut, _event| {
                    let _ = _app.emit("tray-action", "screenshot_full");
                },
            )?;

            app.global_shortcut().on_shortcut(
                h.screenshot_region.parse().unwrap(),
                move |_app, _shortcut, _event| {
                    let _ = _app.emit("tray-action", "screenshot_region");
                },
            )?;

            app.global_shortcut().on_shortcut(
                h.record_screen.parse().unwrap(),
                move |_app, _shortcut, _event| {
                    let _ = _app.emit("tray-action", "record_full");
                },
            )?;

            app.global_shortcut().on_shortcut(
                h.record_region.parse().unwrap(),
                move |_app, _shortcut, _event| {
                    let _ = _app.emit("tray-action", "record_region");
                },
            )?;

            app.global_shortcut().on_shortcut(
                h.stop_recording.parse().unwrap(),
                move |_app, _shortcut, _event| {
                    let _ = _app.emit("tray-action", "stop_recording");
                },
            )?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check 2>&1
```

Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/tray.rs src-tauri/src/main.rs
git commit -m "feat: add tray manager with menu and global hotkeys"
```

---

## Chunk 5: Svelte Frontend

### Task 10: Preview Popup Component

**Files:**
- Modify: `src/App.svelte`
- Create: `src/lib/PreviewPopup.svelte`
- Create: `src/lib/CardPicker.svelte`
- Modify: `src/main.ts`
- Modify: `src/styles.css`

- [ ] **Step 1: Create main.ts entry point**

`src/main.ts`:

```typescript
import App from './App.svelte';
import './styles.css';

const app = new App({
  target: document.getElementById('app')!,
});

export default app;
```

- [ ] **Step 2: Create App.svelte**

`src/App.svelte`:

```svelte
<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';
  import PreviewPopup from './lib/PreviewPopup.svelte';
  import CardPicker from './lib/CardPicker.svelte';

  const appWindow = getCurrentWindow();

  let showPreview = false;
  let showCardPicker = false;
  let captureResult: { file_path: string; filename: string; is_image: boolean } | null = null;

  async function showWindow() {
    await appWindow.show();
    await appWindow.setFocus();
  }

  async function hideWindow() {
    await appWindow.hide();
  }

  onMount(() => {
    listen<string>('tray-action', async (event) => {
      const action = event.payload;

      switch (action) {
        case 'screenshot_full':
          try {
            captureResult = await invoke('take_screenshot', { mode: 'full' });
            showPreview = true;
            await showWindow();
          } catch (e) {
            console.error('Screenshot failed:', e);
          }
          break;

        case 'screenshot_region':
          try {
            captureResult = await invoke('take_screenshot', { mode: 'region' });
            showPreview = true;
            await showWindow();
          } catch (e) {
            console.error('Region screenshot failed:', e);
          }
          break;

        case 'record_full':
          try {
            await invoke('start_recording', { mode: 'full' });
          } catch (e) {
            console.error('Recording failed:', e);
          }
          break;

        case 'record_region':
          try {
            await invoke('start_recording', { mode: 'region' });
          } catch (e) {
            console.error('Recording failed:', e);
          }
          break;

        case 'stop_recording':
          try {
            captureResult = await invoke('stop_recording');
            showPreview = true;
            await showWindow();
          } catch (e) {
            console.error('Stop recording failed:', e);
          }
          break;

        case 'set_card':
          showCardPicker = true;
          await showWindow();
          break;
      }
    });
  });

  async function handleClose() {
    showPreview = false;
    captureResult = null;
    await hideWindow();
  }

  async function handleCardPickerClose() {
    showCardPicker = false;
    await hideWindow();
  }
</script>

{#if showPreview && captureResult}
  <PreviewPopup
    filePath={captureResult.file_path}
    filename={captureResult.filename}
    isImage={captureResult.is_image}
    on:close={handleClose}
  />
{/if}

{#if showCardPicker}
  <CardPicker on:close={handleCardPickerClose} />
{/if}
```

- [ ] **Step 3: Create PreviewPopup.svelte**

`src/lib/PreviewPopup.svelte`:

```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { createEventDispatcher } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';

  export let filePath: string;
  export let filename: string;
  export let isImage: boolean;

  const dispatch = createEventDispatcher();

  let selectedPreset = 'bug_evidence';
  let description = '';
  let activeCard: { key: string; summary: string } | null = null;
  let overrideCard = '';
  let uploading = false;
  let convertingToGif = false;
  let outputFormat: 'mp4' | 'gif' = 'mp4';
  let error = '';
  let presets: [string, string][] = [];

  // Load active card and presets on mount
  async function loadInitialData() {
    try {
      activeCard = await invoke('get_active_card');
      presets = await invoke('get_presets');
      if (presets.length > 0) {
        selectedPreset = presets[0][0];
      }
    } catch (e) {
      console.error('Failed to load initial data:', e);
    }
  }
  loadInitialData();

  // Get the preview URL for the image/video
  $: previewSrc = convertFileSrc(filePath);

  // Determine the issue key to use
  $: issueKey = overrideCard || activeCard?.key || '';

  // Get preset title from loaded presets
  $: presetTitle = presets.find(([key]) => key === selectedPreset)?.[1] || selectedPreset;

  async function handleUploadAndPost() {
    if (!issueKey) {
      error = 'Please select a Jira card first';
      return;
    }

    uploading = true;
    error = '';

    try {
      let uploadPath = filePath;
      let finalIsImage = isImage;

      // Convert to GIF if requested
      if (!isImage && outputFormat === 'gif') {
        convertingToGif = true;
        const gifResult: { file_path: string } = await invoke('convert_to_gif', {
          inputPath: filePath,
        });
        uploadPath = gifResult.file_path;
        finalIsImage = true; // GIF embeds as image
        convertingToGif = false;
      }

      // Upload to R2 and post to Jira (with notifications handled server-side)
      await invoke('upload_and_post', {
        filePath: uploadPath,
        issueKey,
        presetTitle,
        description,
        isImage: finalIsImage,
      });

      dispatch('close');
    } catch (e) {
      error = String(e);
    } finally {
      uploading = false;
      convertingToGif = false;
    }
  }

  function handleSaveLocal() {
    // File is already saved locally by the capture engine
    dispatch('close');
  }
</script>

<div class="popup-overlay">
  <div class="popup">
    <div class="preview">
      {#if isImage}
        <img src={previewSrc} alt="Capture preview" />
      {:else}
        <video src={previewSrc} controls>
          <track kind="captions" />
        </video>
      {/if}
    </div>

    <div class="controls">
      <div class="field">
        <label for="preset">Category</label>
        <select id="preset" bind:value={selectedPreset}>
          {#each presets as [key, title]}
            <option value={key}>{title}</option>
          {/each}
        </select>
      </div>

      <div class="field">
        <label for="card">Jira Card</label>
        <div class="card-display">
          {#if activeCard}
            <span class="active-card">{activeCard.key} - {activeCard.summary}</span>
          {:else}
            <span class="no-card">No card selected</span>
          {/if}
        </div>
        <input
          id="card"
          type="text"
          placeholder="Override: PROJ-123"
          bind:value={overrideCard}
        />
      </div>

      <div class="field">
        <label for="desc">Description</label>
        <textarea id="desc" bind:value={description} placeholder="What is this evidence for?" rows="3"></textarea>
      </div>

      {#if !isImage}
        <div class="field">
          <label for="format">Output Format</label>
          <select id="format" bind:value={outputFormat}>
            <option value="mp4">MP4</option>
            <option value="gif">GIF</option>
          </select>
        </div>
      {/if}

      {#if error}
        <div class="error">{error}</div>
      {/if}

      <div class="actions">
        <button
          class="btn-primary"
          on:click={handleUploadAndPost}
          disabled={uploading}
        >
          {#if convertingToGif}
            Converting to GIF...
          {:else if uploading}
            Uploading...
          {:else}
            Upload & Post
          {/if}
        </button>
        <button class="btn-secondary" on:click={handleSaveLocal} disabled={uploading}>
          Save Local
        </button>
        <button class="btn-cancel" on:click={() => dispatch('close')} disabled={uploading}>
          Cancel
        </button>
      </div>
    </div>
  </div>
</div>
```

- [ ] **Step 4: Create CardPicker.svelte**

`src/lib/CardPicker.svelte`:

```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  let query = '';
  let issues: { key: string; summary: string }[] = [];
  let loading = false;
  let debounceTimer: ReturnType<typeof setTimeout>;

  async function searchIssues() {
    loading = true;
    try {
      issues = await invoke('search_jira_issues', { query });
    } catch (e) {
      console.error('Search failed:', e);
      issues = [];
    } finally {
      loading = false;
    }
  }

  // Search on mount (empty query = recent issues)
  searchIssues();

  function handleInput() {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(searchIssues, 300);
  }

  async function selectCard(issue: { key: string; summary: string }) {
    await invoke('set_active_card', { card: issue });
    dispatch('close');
  }

  function clearCard() {
    invoke('set_active_card', { card: null });
    dispatch('close');
  }
</script>

<div class="popup-overlay">
  <div class="popup card-picker">
    <h3>Select Jira Card</h3>

    <input
      type="text"
      placeholder="Search issues..."
      bind:value={query}
      on:input={handleInput}
    />

    <div class="issue-list">
      {#if loading}
        <div class="loading">Searching...</div>
      {:else if issues.length === 0}
        <div class="empty">No issues found</div>
      {:else}
        {#each issues as issue}
          <button class="issue-item" on:click={() => selectCard(issue)}>
            <span class="issue-key">{issue.key}</span>
            <span class="issue-summary">{issue.summary}</span>
          </button>
        {/each}
      {/if}
    </div>

    <div class="actions">
      <button class="btn-secondary" on:click={clearCard}>Clear Active Card</button>
      <button class="btn-cancel" on:click={() => dispatch('close')}>Cancel</button>
    </div>
  </div>
</div>
```

- [ ] **Step 5: Create styles.css**

`src/styles.css`:

```css
:root {
  --bg: #1e1e2e;
  --surface: #2a2a3e;
  --text: #cdd6f4;
  --text-muted: #a6adc8;
  --primary: #89b4fa;
  --error: #f38ba8;
  --border: #45475a;
  --hover: #313244;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  font-size: 14px;
  color: var(--text);
  background: transparent;
}

.popup-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
}

.popup {
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 20px;
  max-width: 500px;
  width: 90vw;
  max-height: 90vh;
  overflow-y: auto;
}

.preview {
  margin-bottom: 16px;
  border-radius: 8px;
  overflow: hidden;
  background: var(--surface);
}

.preview img,
.preview video {
  width: 100%;
  max-height: 300px;
  object-fit: contain;
}

.controls {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.field label {
  font-size: 12px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

select,
input,
textarea {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 8px 12px;
  color: var(--text);
  font-size: 14px;
}

select:focus,
input:focus,
textarea:focus {
  outline: none;
  border-color: var(--primary);
}

textarea {
  resize: vertical;
}

.card-display {
  font-size: 13px;
  padding: 4px 0;
}

.active-card {
  color: var(--primary);
}

.no-card {
  color: var(--text-muted);
  font-style: italic;
}

.error {
  color: var(--error);
  font-size: 13px;
  padding: 8px;
  background: rgba(243, 139, 168, 0.1);
  border-radius: 6px;
}

.actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

button {
  padding: 8px 16px;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  font-size: 14px;
  transition: opacity 0.2s;
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--primary);
  color: var(--bg);
  flex: 1;
}

.btn-secondary {
  background: var(--surface);
  color: var(--text);
  border: 1px solid var(--border);
}

.btn-cancel {
  background: transparent;
  color: var(--text-muted);
}

button:hover:not(:disabled) {
  opacity: 0.85;
}

/* Card Picker */
.card-picker h3 {
  margin-bottom: 12px;
}

.issue-list {
  margin: 12px 0;
  max-height: 300px;
  overflow-y: auto;
}

.issue-item {
  display: flex;
  gap: 8px;
  width: 100%;
  padding: 10px 12px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
  margin-bottom: 4px;
  text-align: left;
  color: var(--text);
}

.issue-item:hover {
  background: var(--hover);
}

.issue-key {
  color: var(--primary);
  font-weight: 600;
  white-space: nowrap;
}

.issue-summary {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.loading,
.empty {
  padding: 20px;
  text-align: center;
  color: var(--text-muted);
}
```

- [ ] **Step 6: Verify frontend builds**

```bash
npm run build 2>&1
```

Expected: Build succeeds.

- [ ] **Step 7: Commit**

```bash
git add src/
git commit -m "feat: add Svelte frontend with preview popup and card picker"
```

---

## Chunk 6: Integration & Final Wiring

### Task 11: Notifications, Recording Indicator & Tray Tooltip

**Files:**
- Create: `src-tauri/src/notifications.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/tray.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create notifications module**

Create `src-tauri/src/notifications.rs`:

```rust
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

pub fn notify_success(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification()
        .builder()
        .title(title)
        .body(body)
        .show();
}

pub fn notify_error(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification()
        .builder()
        .title(title)
        .body(body)
        .show();
}

pub fn notify_missing_deps(app: &AppHandle, deps: &[String]) {
    let body = format!("Missing: {}. Please install them.", deps.join(", "));
    notify_error(app, "Jira Proofs — Missing Dependencies", &body);
}
```

Add to `src-tauri/src/lib.rs`:

```rust
pub mod notifications;
```

- [ ] **Step 2: Add notification calls to commands.rs**

Add `app: AppHandle` parameter to `upload_to_r2` and `post_to_jira` commands, then call notifications on success/failure. Add these Tauri commands:

```rust
/// Upload and post with notifications
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
    let path = std::path::Path::new(&file_path);

    // Upload to R2
    let url = match r2::upload_file(&state.config.r2, path).await {
        Ok(url) => url,
        Err(e) => {
            crate::notifications::notify_error(&app, "Upload Failed", &e);
            return Err(e);
        }
    };

    // Post to Jira
    match jira::post_comment(
        &state.config.jira,
        &issue_key,
        &preset_title,
        &description,
        &url,
        is_image,
    ).await {
        Ok(()) => {
            crate::notifications::notify_success(
                &app,
                "Posted to Jira",
                &format!("Evidence posted to {}", issue_key),
            );
            Ok(url)
        }
        Err(e) => {
            crate::notifications::notify_error(&app, "Jira Comment Failed", &e);
            Err(e)
        }
    }
}
```

- [ ] **Step 3: Add tray tooltip update and recording icon swap**

Add to `src-tauri/src/tray.rs`:

```rust
use tauri::image::Image;

/// Update tray tooltip to show active card
pub fn update_tooltip(tray: &TrayIcon, card: Option<&str>) {
    let tooltip = match card {
        Some(key) => format!("Jira Proofs — {}", key),
        None => "Jira Proofs".to_string(),
    };
    let _ = tray.set_tooltip(Some(&tooltip));
}

/// Set tray icon to recording indicator
pub fn set_recording_icon(tray: &TrayIcon, recording: bool) {
    // Use a red-tinted icon when recording, default icon otherwise
    // Icons loaded from src-tauri/icons/
    let icon_path = if recording {
        include_bytes!("../icons/tray-icon-recording.png")
    } else {
        include_bytes!("../icons/tray-icon.png")
    };
    if let Ok(icon) = Image::from_bytes(icon_path) {
        let _ = tray.set_icon(Some(icon));
    }
}
```

Note: `set_active_card` was already updated in Task 8 to include `app: AppHandle` and tray tooltip updates.

- [ ] **Step 4: Verify it compiles**

```bash
cd src-tauri && cargo check 2>&1
```

Expected: Compiles without errors.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/notifications.rs src-tauri/src/commands.rs src-tauri/src/tray.rs src-tauri/src/lib.rs
git commit -m "feat: add notifications, recording indicator, and tray tooltip"
```

---

### Task 12: Settings Component

**Files:**
- Create: `src/lib/Settings.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Create Settings.svelte**

`src/lib/Settings.svelte`:

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  // Settings displays the current hotkey configuration (read-only for now).
  // Hotkeys are configured in ~/.config/jira-proofs/config.toml.

  const hotkeys = [
    { label: 'Screenshot (Full)', key: 'Print' },
    { label: 'Screenshot (Region)', key: 'Shift+Print' },
    { label: 'Record Screen', key: 'Ctrl+Alt+R' },
    { label: 'Record Region', key: 'Ctrl+Alt+Shift+R' },
    { label: 'Stop Recording', key: 'Ctrl+Alt+S' },
  ];
</script>

<div class="popup-overlay">
  <div class="popup settings">
    <h3>Hotkey Settings</h3>
    <p class="hint">Edit hotkeys in <code>~/.config/jira-proofs/config.toml</code></p>

    <div class="hotkey-list">
      {#each hotkeys as hotkey}
        <div class="hotkey-item">
          <span class="hotkey-label">{hotkey.label}</span>
          <kbd>{hotkey.key}</kbd>
        </div>
      {/each}
    </div>

    <div class="actions">
      <button class="btn-cancel" on:click={() => dispatch('close')}>Close</button>
    </div>
  </div>
</div>

<style>
  .hint {
    color: var(--text-muted);
    font-size: 12px;
    margin-bottom: 12px;
  }
  code {
    background: var(--surface);
    padding: 2px 6px;
    border-radius: 4px;
  }
  .hotkey-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 16px;
  }
  .hotkey-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: var(--surface);
    border-radius: 6px;
  }
  kbd {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 2px 8px;
    font-family: monospace;
    font-size: 13px;
  }
</style>
```

- [ ] **Step 2: Add settings to App.svelte and tray menu**

In `src/App.svelte`, add:

```svelte
<script lang="ts">
  import Settings from './lib/Settings.svelte';
  let showSettings = false;
</script>
```

Add `'settings'` case to the tray-action listener:

```typescript
case 'settings':
  showSettings = true;
  await showWindow();
  break;
```

Add to template:

```svelte
{#if showSettings}
  <Settings on:close={() => { showSettings = false; hideWindow(); }} />
{/if}
```

In `src-tauri/src/tray.rs`, add a settings menu item before quit:

```rust
let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
```

Add `"settings"` handler in the menu event match:

```rust
"settings" => {
    let _ = app.emit("tray-action", "settings");
}
```

- [ ] **Step 3: Verify frontend builds**

```bash
npm run build 2>&1
```

Expected: Build succeeds.

- [ ] **Step 4: Commit**

```bash
git add src/lib/Settings.svelte src/App.svelte src-tauri/src/tray.rs
git commit -m "feat: add settings panel with hotkey display"
```

---

### Task 13: End-to-End Manual Testing

---

### Task 13: End-to-End Manual Testing

- [ ] **Step 1: Verify system dependencies**

```bash
which scrot ffmpeg slop
```

Expected: All three binaries found. If any missing:
```bash
sudo apt install scrot ffmpeg slop
```

- [ ] **Step 2: Create test config**

```bash
mkdir -p ~/.config/jira-proofs
# Fill in config.toml with real Jira + R2 credentials
```

- [ ] **Step 3: Test screenshot full screen**

1. Launch app: `npm run tauri dev`
2. Press `Print` key (or click "Screenshot (Full)" in tray)
3. Preview popup appears with screenshot
4. Click "Save Local"
5. Verify file saved to `~/Pictures/jira-proofs/`

- [ ] **Step 4: Test screenshot region**

1. Press `Shift+Print`
2. Drag to select region
3. Preview shows cropped screenshot
4. Click "Save Local"

- [ ] **Step 5: Test screen recording**

1. Press `Ctrl+Alt+R` to start recording
2. Verify tray icon changes to recording indicator
3. Wait a few seconds
4. Press `Ctrl+Alt+S` to stop
5. Preview shows video
6. Click "Save Local" — verify MP4 saved to `~/Pictures/jira-proofs/`

- [ ] **Step 6: Test GIF conversion via Upload**

1. Start and stop a recording
2. In preview popup, select "GIF" format
3. Click "Upload & Post" (with valid Jira card)
4. Verify GIF conversion happens before upload
5. Verify GIF file appears in R2 and comment appears on Jira card

- [ ] **Step 7: Test R2 upload + Jira comment**

1. Set active Jira card via tray menu — verify tray tooltip updates
2. Take a screenshot
3. Select "Bug Evidence" preset
4. Type description
5. Click "Upload & Post"
6. Verify: file appears in R2 bucket, comment appears on Jira card
7. Verify: system notification confirms success

- [ ] **Step 8: Test error notifications**

1. Set an invalid Jira card key (e.g. "INVALID-999")
2. Take a screenshot and try "Upload & Post"
3. Verify: system notification shows error

- [ ] **Step 9: Commit any fixes**

```bash
git add -A
git commit -m "fix: adjustments from end-to-end testing"
```

---

### Task 14: Add Tray Icon Assets

**Files:**
- Create: `src-tauri/icons/tray-icon.png`
- Create: `src-tauri/icons/tray-icon-recording.png`

- [ ] **Step 1: Create a simple tray icon**

Create a 32x32 PNG icon for the tray. Can use a simple camera/proof icon. For now, use the default Tauri icon or create a minimal one.

- [ ] **Step 2: Add icon to tray builder**

In `src-tauri/src/tray.rs`, update `TrayIconBuilder`:

```rust
let tray = TrayIconBuilder::new()
    .icon(app.default_window_icon().unwrap().clone())
    .menu(&menu)
    .tooltip("Jira Proofs")
    // ...
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/icons/ src-tauri/src/tray.rs
git commit -m "feat: add tray icon"
```

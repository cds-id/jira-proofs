# Jira Proofs — Design Spec

**Date:** 2026-03-10
**Status:** Approved

## Overview

A Tauri v2 system tray application for Linux Mint (Cinnamon/X11) that captures screenshots and screen recordings, uploads them to Cloudflare R2, and posts preset comments (bug evidence, work evidence) to Jira Cloud cards.

## Tech Stack

- **Framework:** Tauri v2 (Rust backend + webview frontend)
- **Frontend:** Svelte + TypeScript
- **Backend:** Rust
- **Capture:** `scrot` (screenshots), `ffmpeg` (recording + GIF conversion)
- **R2:** `aws-sdk-s3` Rust crate (S3-compatible API)
- **Jira:** Jira Cloud REST API v3
- **Target:** Linux Mint, Cinnamon DE, Mutter/Muffin, X11

## Architecture

```
┌─────────────────────────────────────────────┐
│              System Tray Icon                │
│  ┌─────────────────────────────────────┐     │
│  │ Menu:                               │     │
│  │  - Screenshot (Full)                │     │
│  │  - Screenshot (Region)              │     │
│  │  - Record Screen / Record Region    │     │
│  │  - Stop Recording                   │     │
│  │  - Set Active Jira Card             │     │
│  │  - Settings (hotkeys)               │     │
│  │  - Quit                             │     │
│  └─────────────────────────────────────┘     │
└──────────────────┬──────────────────────────┘
                   │ capture event
                   ▼
┌─────────────────────────────────────────────┐
│            Capture Preview Popup             │
│  ┌─────────┐  Category: [Bug ▼]            │
│  │ preview  │  Card: PROJ-123 (sticky)      │
│  │  image   │  Description: [editable]      │
│  │          │                               │
│  └─────────┘  [Upload & Post] [Save Local]  │
└──────────────────┬──────────────────────────┘
                   │ user confirms
                   ▼
┌──────────────┐      ┌──────────────┐
│  R2 Upload   │─────▶│  Jira Comment │
│  (S3 API)    │ URL  │  (REST API)   │
└──────────────┘      └──────────────┘
```

### Components

1. **Tray Manager** (Rust) — system tray icon + menu, global hotkey registration
2. **Capture Engine** (Rust) — shells out to `scrot` for screenshots, `ffmpeg` for recording/GIF
3. **Preview Popup** (Svelte) — small Tauri window that appears after capture
4. **R2 Client** (Rust) — uploads assets to Cloudflare R2 via S3-compatible API
5. **Jira Client** (Rust) — posts comments with preset templates to Jira Cloud
6. **Config Manager** (Rust) — reads `~/.config/jira-proofs/config.toml`

## Config

File: `~/.config/jira-proofs/config.toml` (created with `chmod 600` — contains secrets)

```toml
[jira]
base_url = "https://yourteam.atlassian.net"
email = "you@example.com"
api_token = "your-jira-api-token"
default_project = "PROJ"

[r2]
account_id = "your-cf-account-id"
access_key_id = "your-r2-access-key"
secret_access_key = "your-r2-secret"
bucket = "jira-proofs"
public_url = "https://assets.yourdomain.com"

[hotkeys]
screenshot_full = "Print"
screenshot_region = "Shift+Print"
record_screen = "Ctrl+Alt+R"
record_region = "Ctrl+Alt+Shift+R"
stop_recording = "Ctrl+Alt+S"

[storage]
local_dir = "~/Pictures/jira-proofs"

[presets]
bug_evidence = "🐛 **Bug Evidence**\n\n{description}\n\n![]({url})"
work_evidence = "✅ **Work Evidence**\n\n{description}\n\n![]({url})"
```

## Data Flow

1. User triggers capture (hotkey or tray menu)
2. Capture engine runs `scrot`/`ffmpeg`, saves file to `local_dir` with timestamp name
3. Preview popup opens with thumbnail, category selector, card picker, description field
4. User picks category + optional description, clicks "Upload & Post" or "Save Local"
5. If upload: Rust uploads file to R2 → gets public URL → posts Jira comment using preset template
6. System notification confirms success/failure

## Capture Engine

### Screenshots
- Full screen: `scrot /tmp/jira-proofs-{timestamp}.png`
- Region select: `scrot -s /tmp/jira-proofs-{timestamp}.png`
- Files saved to `local_dir` with naming: `{YYYY-MM-DD}_{HHmmss}_{type}.png`

### Screen Recording
- Full screen: `ffmpeg -f x11grab -i :0.0 -c:v libx264 -preset ultrafast output.mp4`
- Region: same with `-video_size WxH -grab_x X -grab_y Y` (region selected via `slop` — lightweight X11 region selector)
- GIF conversion: `ffmpeg -i output.mp4 -vf "fps=15,scale=640:-1" output.gif`
- User chooses MP4 or GIF in preview popup before uploading
- Stop recording via hotkey or tray menu
- Tray icon changes indicator while recording is active

### System Dependencies
- `scrot` — screenshot capture
- `ffmpeg` — recording + GIF conversion
- `slop` — interactive region selection for recording
- App checks for these on startup and warns if missing

## R2 Integration

- Uses `aws-sdk-s3` Rust crate (R2 is S3-compatible)
- Endpoint: `https://{account_id}.r2.cloudflarestorage.com`
- Upload path: `{bucket}/captures/{YYYY-MM-DD}/{filename}`
- Returns public URL: `{public_url}/captures/{YYYY-MM-DD}/{filename}`
- Content-Type set based on file extension (image/png, video/mp4, image/gif)
- R2 bucket must have public access enabled via custom domain, or use presigned URLs for private buckets

## Jira Integration

- Jira Cloud REST API v3
- Auth: Basic auth (email + API token)
- Post comment: `POST /rest/api/3/issue/{issueKey}/comment`
- Comment body uses Atlassian Document Format (ADF)
- Images: ADF `mediaSingle` node with inline image
- Videos/GIFs: link to R2 URL (Jira doesn't embed video natively)

### Card Selection
- JQL query: `project = {default_project} AND status != Done ORDER BY updated DESC`
- Returns top 20 results in searchable dropdown
- Shows: issue key + summary (e.g. `PROJ-123 - Fix login bug`)
- "Sticky" card mode: active card persists until changed, shown in tray tooltip
- Preview popup allows one-off card override

### Comment Template Rendering
- Config stores human-readable templates with `{url}` and `{description}` placeholders
- Rust backend programmatically constructs ADF JSON from these templates at post time
- The Markdown-like syntax in config is for readability only — it is parsed and converted to ADF nodes

### Comment Presets
- **Bug evidence:** bug report with description + attached screenshot/recording
- **Work evidence:** work completion proof with description + attachment
- Templates configurable in `config.toml` with `{url}` and `{description}` placeholders

## Project Structure

```
jira-proofs/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs              # app entry, tray setup
│   │   ├── tray.rs              # tray icon, menu, hotkey registration
│   │   ├── capture.rs           # scrot/ffmpeg shell commands
│   │   ├── r2.rs                # R2 upload client
│   │   ├── jira.rs              # Jira REST API client
│   │   ├── config.rs            # config.toml parsing
│   │   └── commands.rs          # Tauri IPC commands (frontend ↔ backend)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── App.svelte               # root component
│   ├── lib/
│   │   ├── PreviewPopup.svelte  # capture preview + category + card picker
│   │   ├── CardPicker.svelte    # Jira card search dropdown
│   │   └── Settings.svelte      # hotkey configuration
│   ├── main.ts
│   └── styles.css
├── package.json
└── svelte.config.js
```

## Error Handling

- **Missing dependencies:** On startup, check `scrot` and `ffmpeg` — show system notification if missing
- **R2 upload failure:** Error notification, file stays in local dir, retry from tray menu
- **Jira API failure:** Error notification with reason, capture saved locally
- **Config missing/invalid:** On first launch, create template `config.toml`, notify user to fill it in
- **Recording in progress:** Prevent starting second recording, tray shows "Stop Recording"

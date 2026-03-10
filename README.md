# Jira Proofs

A Tauri v2 system tray application for Linux Mint that captures screenshots and screen recordings, uploads them to Cloudflare R2, and posts evidence comments to Jira Cloud cards.

## Features

- **Screenshot capture** — full screen or region select (via `scrot`)
- **Screen recording** — full screen or region, MP4 or GIF output (via `ffmpeg` + `slop`)
- **R2 upload** — uploads captures to Cloudflare R2 via S3-compatible API
- **Jira integration** — posts preset comments (bug evidence, work evidence) with embedded media to Jira Cloud
- **System tray** — runs in background with configurable global hotkeys
- **Sticky card** — set an active Jira card, all captures attach to it until changed

## Requirements

- Linux Mint (Cinnamon DE / X11)
- [Bun](https://bun.sh/) (package manager)
- [Rust](https://rustup.rs/) (for Tauri backend)
- System dependencies:
  ```bash
  sudo apt install scrot ffmpeg slop
  ```

## Setup

```bash
# Install dependencies
bun install

# Create config (first run creates template)
bun run tauri dev

# Edit config with your credentials
nano ~/.config/jira-proofs/config.toml
```

## Configuration

Config file: `~/.config/jira-proofs/config.toml`

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
bug_evidence = "Bug Evidence: {description} {url}"
work_evidence = "Work Evidence: {description} {url}"
```

## Default Hotkeys

| Action | Shortcut |
|--------|----------|
| Screenshot (Full) | `Print` |
| Screenshot (Region) | `Shift+Print` |
| Record Screen | `Ctrl+Alt+R` |
| Record Region | `Ctrl+Alt+Shift+R` |
| Stop Recording | `Ctrl+Alt+S` |

## Development

```bash
# Dev mode
bun run tauri dev

# Build
bun run tauri build

# Run tests
cd src-tauri && cargo test
```

## Tech Stack

- **Tauri v2** — Rust backend + webview frontend
- **Svelte + TypeScript** — frontend UI
- **Rust** — capture engine, R2/Jira clients, system tray
- **scrot** — screenshot capture
- **ffmpeg** — screen recording + GIF conversion
- **slop** — interactive region selection

## License

MIT

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
                bug_evidence: "Bug Evidence\n\n{description}\n\n![]({url})".into(),
                work_evidence: "Work Evidence\n\n{description}\n\n![]({url})".into(),
            },
        }
    }
}

pub fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("jira-proofs")
        .join("config.toml")
}

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
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        fs::set_permissions(path, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }
    Ok(())
}

pub fn render_template(template: &str, description: &str, url: &str) -> String {
    template
        .replace("{description}", description)
        .replace("{url}", url)
}

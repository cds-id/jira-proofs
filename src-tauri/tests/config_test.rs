use app_lib::config::AppConfig;

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
    let rendered = app_lib::config::render_template(
        template,
        "Login broken",
        "https://assets.example.com/img.png",
    );
    assert_eq!(rendered, "Bug: Login broken\nhttps://assets.example.com/img.png");
}

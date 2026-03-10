use app_lib::r2::{build_object_key, build_public_url, content_type_for_extension};

#[test]
fn test_build_object_key() {
    let key = build_object_key("2026-03-10_143022_screenshot_full.png");
    assert!(key.starts_with("captures/"));
    assert!(key.ends_with("/2026-03-10_143022_screenshot_full.png"));
    let parts: Vec<&str> = key.split('/').collect();
    assert_eq!(parts.len(), 3);
}

#[test]
fn test_build_public_url() {
    let url = build_public_url("https://assets.example.com", "captures/2026-03-10/test.png");
    assert_eq!(url, "https://assets.example.com/captures/2026-03-10/test.png");
}

#[test]
fn test_content_type_png() { assert_eq!(content_type_for_extension("png"), "image/png"); }

#[test]
fn test_content_type_mp4() { assert_eq!(content_type_for_extension("mp4"), "video/mp4"); }

#[test]
fn test_content_type_gif() { assert_eq!(content_type_for_extension("gif"), "image/gif"); }

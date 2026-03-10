use app_lib::jira::{build_auth_header, build_adf_comment, build_adf_image_comment, build_adf_link_comment};

#[test]
fn test_build_auth_header() {
    let header = build_auth_header("user@example.com", "token123");
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
    let adf = build_adf_comment("Bug Evidence", "Login button broken", "https://r2.example.com/img.png", true);
    let json = serde_json::to_string(&adf).unwrap();
    assert!(json.contains("Bug Evidence"));
    assert!(json.contains("Login button broken"));
}

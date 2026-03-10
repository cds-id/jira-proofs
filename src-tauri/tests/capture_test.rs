use app_lib::capture::{build_screenshot_command, CaptureMode, generate_filename};

#[test]
fn test_generate_filename_screenshot() {
    let name = generate_filename("screenshot_full", "png");
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

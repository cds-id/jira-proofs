use app_lib::capture::{build_screenshot_command, CaptureMode, generate_filename};
use app_lib::capture::{build_record_command, build_gif_convert_command, build_slop_command, parse_slop_output};

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
    let region = app_lib::capture::Region { x: 100, y: 200, width: 800, height: 600 };
    let (cmd, args) = build_record_command(CaptureMode::Region, "/tmp/test.mp4", Some(region));
    assert_eq!(cmd, "ffmpeg");
    assert!(args.contains(&"800x600".to_string()));
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

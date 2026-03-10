use app_lib::deps::{check_dependency, DependencyStatus};

#[test]
fn test_check_existing_binary() {
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
    let results = app_lib::deps::check_all();
    assert_eq!(results.len(), 3); // scrot, ffmpeg, slop
}

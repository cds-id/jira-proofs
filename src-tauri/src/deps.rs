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
    REQUIRED_DEPS.iter().map(|dep| (dep.to_string(), check_dependency(dep))).collect()
}

pub fn missing_deps() -> Vec<String> {
    check_all().into_iter()
        .filter_map(|(name, status)| match status {
            DependencyStatus::Missing(_) => Some(name),
            DependencyStatus::Found(_) => None,
        })
        .collect()
}

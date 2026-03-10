use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

pub fn notify_success(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

pub fn notify_error(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

pub fn notify_missing_deps(app: &AppHandle, deps: &[String]) {
    let body = format!("Missing: {}. Please install them.", deps.join(", "));
    notify_error(app, "Jira Proofs — Missing Dependencies", &body);
}

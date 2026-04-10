use liveletters_rust_backend_app::app_name;

#[cfg(feature = "tauri-runtime")]
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn app_library_is_available() {
    assert_eq!(app_name(), "liveletters-rust-backend-app");
}

#[cfg(feature = "tauri-runtime")]
#[test]
fn runtime_log_helpers_use_stable_paths_and_format() {
    let home_dir = std::env::temp_dir().join(format!(
        "liveletters-runtime-home-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let log_dir = liveletters_rust_backend_app::runtime_log_dir_for_home(&home_dir);
    let log_dir_text = log_dir.to_string_lossy();
    assert!(log_dir_text.ends_with(".liveletters/runtime-logs"));
    assert!(liveletters_rust_backend_app::runtime_log_dir().is_none());

    let line = liveletters_rust_backend_app::runtime_log_line(
        "window.onerror",
        "boom\nnext",
        Some("stack\ntrace"),
        Some("main.js"),
        Some("10:20"),
    );

    assert!(line.contains("[kind=window.onerror]"));
    assert!(line.contains("[message=boom next]"));
    assert!(line.contains("[stack=stack trace]"));
    assert!(line.contains("[source=main.js]"));
    assert!(line.contains("[location=10:20]"));

    let _ = fs::remove_dir_all(home_dir);
}

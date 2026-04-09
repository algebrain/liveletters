use liveletters_rust_backend_app::app_name;

#[test]
fn app_library_is_available() {
    assert_eq!(app_name(), "liveletters-rust-backend-app");
}

#[cfg(feature = "tauri-runtime")]
#[test]
fn runtime_log_helpers_use_stable_paths_and_format() {
    let log_dir = liveletters_rust_backend_app::runtime_log_dir();
    let log_dir_text = log_dir.to_string_lossy();
    assert!(log_dir_text.ends_with(".docs/runtime-logs"));

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
}

use liveletters_rust_backend_app::app_name;

#[test]
fn app_library_is_available() {
    assert_eq!(app_name(), "liveletters-rust-backend-app");
}


#[cfg(feature = "tauri-runtime")]
fn main() {
    liveletters_rust_backend_app::build_tauri_app()
        .expect("tauri app should bootstrap")
        .run(tauri::generate_context!())
        .expect("tauri app should run");
}

#[cfg(not(feature = "tauri-runtime"))]
fn main() {
    eprintln!(
        "liveletters-rust-backend-app was built without `tauri-runtime`; use `cargo run --features tauri-runtime` in an environment with Tauri system dependencies"
    );
}

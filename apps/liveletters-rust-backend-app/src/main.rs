#[cfg(feature = "tauri-runtime")]
fn main() {
    let options = liveletters_rust_backend_app::RuntimeOptions::from_args(
        std::env::args_os().skip(1),
    )
    .unwrap_or_else(|error| {
        eprintln!("invalid runtime arguments: {error}");
        std::process::exit(2);
    });

    options
        .apply_to_process_environment()
        .unwrap_or_else(|error| {
            eprintln!("failed to prepare --home-dir: {error}");
            std::process::exit(2);
        });

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

fn main() {
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!("cargo:rerun-if-changed=capabilities");
    println!("cargo:rerun-if-changed=permissions");

    if std::env::var_os("CARGO_FEATURE_TAURI_RUNTIME").is_some() {
        tauri_build::build()
    }
}

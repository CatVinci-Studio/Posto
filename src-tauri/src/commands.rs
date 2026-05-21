#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello from Rust, {name}!")
}

#[tauri::command]
pub fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello from Rust, {name}!")
}

#[tauri::command]
pub fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[tauri::command]
pub async fn list_accounts(
    state: tauri::State<'_, sqlx::SqlitePool>,
) -> Result<Vec<crate::storage::Account>, String> {
    crate::storage::queries::list_accounts(&state)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_external(url: String, app: tauri::AppHandle) -> Result<(), String> {
    tauri_plugin_opener::OpenerExt::opener(&app)
        .open_url(&url, None::<&str>)
        .map_err(|e| e.to_string())
}

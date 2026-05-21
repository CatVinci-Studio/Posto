use tauri::Manager;
use tracing_subscriber::EnvFilter;

mod commands;
mod error;

// Feature modules (filled in by parallel work streams)
pub mod accounts;
pub mod agents;
pub mod llm;
pub mod memory;
pub mod storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,retposto_lib=debug")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            let pool = tauri::async_runtime::block_on(crate::storage::init(app.handle()))?;
            app.manage(pool);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::app_version,
            commands::list_accounts,
            accounts::commands::detect_provider,
            accounts::commands::list_providers,
            accounts::commands::get_provider_config,
            accounts::commands::test_imap_login,
            llm::commands::set_openai_api_key,
            llm::commands::has_openai_api_key,
            llm::commands::clear_openai_api_key,
            llm::commands::test_openai_completion,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

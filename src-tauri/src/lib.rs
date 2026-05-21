use std::sync::Arc;

use tauri::Manager;
use tauri_plugin_deep_link::DeepLinkExt;
use tracing_subscriber::EnvFilter;

mod commands;
mod error;

// Feature modules
pub mod accounts;
pub mod agents;
pub mod llm;
pub mod memory;
pub mod messages;
pub mod storage;
pub mod sync;
pub mod translation;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,posto_lib=debug")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            // ---- Storage ----
            let pool = tauri::async_runtime::block_on(crate::storage::init(app.handle()))?;
            app.manage(pool.clone());

            // ---- OAuth ----
            let oauth_flow = Arc::new(accounts::oauth::OAuthFlow::from_env());
            app.manage(oauth_flow.clone());
            app.manage(accounts::oauth::PendingStore::default());

            // ---- Sync engine + background polling + IDLE workers ----
            let engine = Arc::new(sync::SyncEngine::new(
                pool,
                app.handle().clone(),
                oauth_flow,
            ));
            let engine_for_loop = engine.clone();
            tauri::async_runtime::spawn(async move {
                engine_for_loop.run_polling_loop().await;
            });
            let engine_for_idle = engine.clone();
            tauri::async_runtime::spawn(async move {
                // Allow startup to finish before opening long-lived connections.
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                engine_for_idle.spawn_idle_workers().await;
            });
            app.manage(engine);

            // ---- Deep-link OAuth callback ----
            let app_handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    let url_str = url.to_string();
                    if url_str.starts_with("posto://oauth/callback") {
                        let handle = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            let flow = handle.state::<Arc<accounts::oauth::OAuthFlow>>();
                            let pending = handle.state::<accounts::oauth::PendingStore>();
                            if let Err(e) = accounts::oauth::commands::handle_oauth_callback(
                                url_str,
                                flow,
                                pending,
                                handle.clone(),
                            )
                            .await
                            {
                                tracing::warn!("OAuth callback failed: {e}");
                            }
                        });
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Built-ins
            commands::greet,
            commands::app_version,
            commands::list_accounts,
            commands::open_external,
            // Accounts / providers
            accounts::commands::detect_provider,
            accounts::commands::list_providers,
            accounts::commands::get_provider_config,
            accounts::commands::test_imap_login,
            // OAuth
            accounts::oauth::commands::begin_oauth_login,
            accounts::oauth::commands::handle_oauth_callback,
            accounts::oauth::commands::refresh_oauth_tokens,
            accounts::oauth::commands::has_oauth_tokens,
            accounts::oauth::commands::clear_oauth_tokens,
            // Sync
            sync::commands::trigger_sync,
            sync::commands::get_sync_status,
            sync::commands::save_account_password,
            sync::commands::add_password_account,
            sync::commands::add_custom_imap_account,
            sync::commands::list_folders,
            sync::commands::list_attachments,
            sync::commands::open_attachment,
            // LLM
            llm::commands::set_openai_api_key,
            llm::commands::has_openai_api_key,
            llm::commands::clear_openai_api_key,
            llm::commands::test_openai_completion,
            // Agents
            agents::commands::trigger_agent_pipeline,
            agents::commands::list_agent_runs,
            // Messages (inbox / actions)
            messages::commands::list_inbox_items,
            messages::commands::mark_read,
            messages::commands::flag_message,
            messages::commands::archive_message,
            messages::commands::delete_message,
            messages::commands::create_task_from_message,
            // Memory
            memory::commands::list_memories,
            memory::commands::pin_memory,
            memory::commands::delete_memory,
            memory::commands::add_memory_manual,
            memory::commands::search_memories_semantic,
            // Translation
            translation::commands::translate_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

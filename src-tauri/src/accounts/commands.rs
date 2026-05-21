use crate::accounts::imap_password::{self, ImapAuth, ImapCredentials};
use crate::accounts::provider::{Encryption, ImapConfig, ProviderConfig, ProviderKind};
use crate::accounts::provider_catalog;

/// Detect which provider an email address belongs to.
/// Returns `None` for unknown / generic domains.
#[tauri::command]
pub fn detect_provider(email: String) -> Option<ProviderKind> {
    provider_catalog::detect_by_domain(&email)
}

/// Return full static configs for every provider in picker order.
#[tauri::command]
pub fn list_providers() -> Vec<ProviderConfig> {
    provider_catalog::all_for_picker()
        .into_iter()
        .map(provider_catalog::config)
        .collect()
}

/// Return the static config for a single provider kind.
#[tauri::command]
pub fn get_provider_config(kind: ProviderKind) -> ProviderConfig {
    provider_catalog::config(kind)
}

/// Attempt an IMAP login with the given credentials and return immediately.
/// Used by the onboarding flow to verify that a password / auth-code is correct.
#[tauri::command]
pub async fn test_imap_login(
    host: String,
    port: u16,
    encryption: Encryption,
    username: String,
    password: String,
) -> Result<(), String> {
    let imap_config = ImapConfig {
        host,
        port,
        encryption,
    };
    let auth = ImapAuth::Password(ImapCredentials { username, password });
    imap_password::test_connection(&imap_config, &auth)
        .await
        .map_err(|e| e.to_string())
}

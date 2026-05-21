use std::sync::Arc;

use sqlx::SqlitePool;
use tauri::State;

use crate::accounts::provider::{Encryption, ProviderKind as AccountProviderKind};
use crate::accounts::provider_catalog;
use crate::storage::models::{Account, Attachment, Folder, ProviderKind};
use crate::storage::queries;

use super::credentials;
use super::engine::{AccountSyncReport, SyncEngine, SyncStatus};

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Manually trigger a sync. If `account_id` is `Some`, sync only that account;
/// otherwise sync all active accounts.
#[tauri::command]
pub async fn trigger_sync(
    account_id: Option<i64>,
    engine: State<'_, Arc<SyncEngine>>,
) -> Result<Vec<AccountSyncReport>, String> {
    match account_id {
        Some(id) => {
            let report = engine
                .sync_account(id)
                .await
                .map_err(|e| e.to_string())?;
            Ok(vec![report])
        }
        None => Ok(engine.sync_all().await),
    }
}

/// Return the cached sync status for a given account, or `null` if no sync
/// has been run yet.
#[tauri::command]
pub async fn get_sync_status(
    account_id: i64,
    engine: State<'_, Arc<SyncEngine>>,
) -> Result<Option<SyncStatus>, String> {
    Ok(engine.current_status(account_id).await)
}

/// Persist an IMAP password in the system keychain for `email`.
#[tauri::command]
pub fn save_account_password(email: String, password: String) -> Result<(), String> {
    credentials::save_imap_password(&email, &password).map_err(|e| e.to_string())
}

/// Add a new password-auth account (IMAP+password / app-password / auth-code).
///
/// Steps:
/// 1. Resolve IMAP/SMTP config from provider catalog (fails for `generic_imap` —
///    use a future `add_custom_imap_account` command that accepts explicit config).
/// 2. Save password in system keychain.
/// 3. Insert account row and return the new `account_id`.
#[tauri::command]
pub async fn add_password_account(
    provider: ProviderKind,
    email: String,
    password: String,
    display_name: Option<String>,
    pool: State<'_, SqlitePool>,
) -> Result<i64, String> {
    // 1. Resolve config from catalog. GenericImap has no fixed host, so reject it
    //    here and ask the caller to supply explicit IMAP config instead.
    let account_provider = map_provider_kind(&provider);
    if matches!(account_provider, AccountProviderKind::GenericImap) {
        return Err(
            "generic_imap requires explicit imap_host/port; \
             use the add_custom_imap_account command instead"
                .to_string(),
        );
    }

    let cfg = provider_catalog::config(account_provider);
    let imap = cfg.imap.ok_or_else(|| {
        format!(
            "provider '{}' has no IMAP configuration",
            provider.as_str()
        )
    })?;
    let smtp = cfg.smtp;

    // 2. Save password to system keychain.
    credentials::save_imap_password(&email, &password).map_err(|e| e.to_string())?;

    // 3. Insert account row.
    let now = chrono::Utc::now().timestamp();
    let encryption_str = format!("{:?}", imap.encryption).to_lowercase();
    let smtp_encryption_str = smtp
        .as_ref()
        .map(|s| format!("{:?}", s.encryption).to_lowercase());

    let account = Account {
        id: None,
        provider: Some(provider.as_str().to_string()),
        email: Some(email.clone()),
        display_name,
        oauth_token_ref: None,
        imap_host: Some(imap.host),
        imap_port: Some(imap.port as i64),
        imap_encryption: Some(encryption_str),
        smtp_host: smtp.as_ref().map(|s| s.host.clone()),
        smtp_port: smtp.as_ref().map(|s| s.port as i64),
        smtp_encryption: smtp_encryption_str,
        status: Some("active".to_string()),
        created_at: now,
        updated_at: now,
    };

    let account_id = queries::insert_account(&pool, &account)
        .await
        .map_err(|e| e.to_string())?;

    Ok(account_id)
}

/// Add a custom IMAP account (generic_imap) — user supplies explicit IMAP/SMTP
/// configuration. SMTP fields are optional; if omitted, sending is disabled.
#[tauri::command]
pub async fn add_custom_imap_account(
    email: String,
    password: String,
    display_name: Option<String>,
    imap_host: String,
    imap_port: u16,
    imap_encryption: String, // "tls" | "starttls" | "none"
    smtp_host: Option<String>,
    smtp_port: Option<u16>,
    smtp_encryption: Option<String>,
    pool: State<'_, SqlitePool>,
) -> Result<i64, String> {
    fn parse_encryption(s: &str) -> Result<Encryption, String> {
        match s.to_ascii_lowercase().as_str() {
            "tls" => Ok(Encryption::Tls),
            "starttls" => Ok(Encryption::StartTls),
            "none" => Ok(Encryption::None),
            other => Err(format!("invalid encryption '{other}'")),
        }
    }

    let imap_enc = parse_encryption(&imap_encryption)?;
    let smtp_enc = smtp_encryption.as_deref().map(parse_encryption).transpose()?;

    credentials::save_imap_password(&email, &password).map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().timestamp();
    let account = Account {
        id: None,
        provider: Some("generic_imap".to_string()),
        email: Some(email.clone()),
        display_name,
        oauth_token_ref: None,
        imap_host: Some(imap_host),
        imap_port: Some(imap_port as i64),
        imap_encryption: Some(format!("{:?}", imap_enc).to_lowercase()),
        smtp_host,
        smtp_port: smtp_port.map(|p| p as i64),
        smtp_encryption: smtp_enc.map(|e| format!("{:?}", e).to_lowercase()),
        status: Some("active".to_string()),
        created_at: now,
        updated_at: now,
    };

    queries::insert_account(&pool, &account)
        .await
        .map_err(|e| e.to_string())
}

/// Return all folders stored locally for the given account.
#[tauri::command]
pub async fn list_folders(
    account_id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<Vec<Folder>, String> {
    queries::list_folders(pool.inner().clone(), account_id)
        .await
        .map_err(|e| e.to_string())
}

/// List attachments stored locally for a message.
#[tauri::command]
pub async fn list_attachments(
    message_id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<Vec<Attachment>, String> {
    queries::list_attachments(&pool, message_id)
        .await
        .map_err(|e| e.to_string())
}

/// Open an attachment in the OS default handler via tauri-plugin-opener.
#[tauri::command]
pub async fn open_attachment(
    attachment_id: i64,
    pool: State<'_, SqlitePool>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let att = queries::get_attachment(&pool, attachment_id)
        .await
        .map_err(|e| e.to_string())?;
    let path = att
        .blob_path
        .ok_or_else(|| "attachment has no local blob_path".to_string())?;
    tauri_plugin_opener::OpenerExt::opener(&app)
        .open_path(&path, None::<&str>)
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Bridge between the storage `ProviderKind` (used in DB models) and the
/// accounts `ProviderKind` (used in provider catalog).  They are structurally
/// identical; we map by name to avoid a circular dependency.
fn map_provider_kind(pk: &ProviderKind) -> AccountProviderKind {
    match pk {
        ProviderKind::Gmail => AccountProviderKind::Gmail,
        ProviderKind::Outlook => AccountProviderKind::Outlook,
        ProviderKind::ICloud => AccountProviderKind::ICloud,
        ProviderKind::Qq => AccountProviderKind::Qq,
        ProviderKind::Mail163 => AccountProviderKind::Mail163,
        ProviderKind::GenericImap => AccountProviderKind::GenericImap,
    }
}

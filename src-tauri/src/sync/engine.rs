use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::accounts::imap_password::{self, ImapCredentials};
use crate::accounts::provider::{Encryption, ImapConfig};
use crate::accounts::provider_catalog;
use crate::error::{AppError, AppResult};
use crate::storage::models::{Account, Folder, FolderKind, Message, ProviderKind};
use crate::storage::queries;

pub const MAX_PER_FOLDER: u32 = 50;
pub const POLL_INTERVAL_SECS: u64 = 60;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct SyncStatus {
    pub account_id: i64,
    pub account_email: String,
    pub folder_count: u32,
    pub messages_fetched: u32,
    pub messages_new: u32,
    pub finished_at: Option<i64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccountSyncReport {
    pub account_id: i64,
    pub folders: Vec<String>,
    pub new_messages: u32,
    pub duration_ms: u64,
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

pub struct SyncEngine {
    pool: SqlitePool,
    app: AppHandle,
    /// Last reported status per account_id.
    statuses: Arc<Mutex<HashMap<i64, SyncStatus>>>,
}

impl SyncEngine {
    pub fn new(pool: SqlitePool, app: AppHandle) -> Self {
        Self {
            pool,
            app,
            statuses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Return the latest cached SyncStatus for a given account, if any.
    pub async fn current_status(&self, account_id: i64) -> Option<SyncStatus> {
        self.statuses.lock().await.get(&account_id).cloned()
    }

    /// Sync a single account. Returns an `AccountSyncReport` on success.
    pub async fn sync_account(&self, account_id: i64) -> AppResult<AccountSyncReport> {
        let start = Instant::now();

        // 1. Load account row.
        let account = queries::get_account(self.pool.clone(), account_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("account {account_id} not found")))?;

        let email = account
            .email
            .clone()
            .unwrap_or_else(|| format!("account_{account_id}"));

        info!("sync_account: starting sync for {email}");

        // 2. Detect provider and resolve ImapConfig.
        let imap_cfg = resolve_imap_config(&account)?;

        // 3. Load IMAP password from keyring.
        let password = super::credentials::load_imap_password(&email)?
            .ok_or_else(|| {
                // TODO: XOAUTH2 — for OAuth-only providers (Gmail, Outlook) the
                // password slot will be empty. Implement XOAUTH2 SASL here once
                // the OAuth token store is wired up.
                AppError::Auth(format!(
                    "no IMAP password stored for {email} \
                     (OAuth / XOAUTH2 not yet implemented)"
                ))
            })?;

        let creds = ImapCredentials {
            username: email.clone(),
            password,
        };

        // 4. Fetch folder list and upsert each into `folders` table.
        let folder_infos = imap_password::fetch_folders(&imap_cfg, &creds).await?;

        let mut folder_names: Vec<String> = Vec::with_capacity(folder_infos.len());
        let mut folder_id_map: HashMap<String, i64> = HashMap::new();

        for fi in &folder_infos {
            let kind = infer_folder_kind(&fi.attributes);
            let folder = Folder {
                id: None,
                account_id,
                name: Some(fi.name.clone()),
                kind: Some(kind.to_string()),
                imap_path: Some(fi.imap_path.clone()),
                uid_validity: None,
                uid_next: None,
            };
            let fid = queries::upsert_folder(&self.pool, &folder).await?;
            folder_names.push(fi.imap_path.clone());
            // After upsert we need the stable id — fetch it back.
            let stored_id = queries::get_folder_id_by_path(
                self.pool.clone(),
                account_id,
                &fi.imap_path,
            )
            .await?
            .unwrap_or(fid);
            folder_id_map.insert(fi.imap_path.clone(), stored_id);
        }

        // 5-8. For each folder, fetch UIDs, diff, insert new messages.
        let mut messages_fetched: u32 = 0;
        let mut messages_new: u32 = 0;
        let folder_count = folder_infos.len() as u32;

        for fi in &folder_infos {
            let folder_id = match folder_id_map.get(&fi.imap_path) {
                Some(id) => *id,
                None => continue,
            };

            // Fetch recent UIDs from server.
            let server_uids =
                match imap_password::fetch_recent_uids(&imap_cfg, &creds, &fi.imap_path, MAX_PER_FOLDER)
                    .await
                {
                    Ok(uids) => uids,
                    Err(e) => {
                        warn!("fetch_recent_uids failed for {}: {e}", fi.imap_path);
                        continue;
                    }
                };

            // Load UIDs already in the DB for this folder.
            let existing_uids: HashSet<u32> =
                queries::list_existing_uids(self.pool.clone(), folder_id).await?;

            // Compute diff — only fetch UIDs we don't have.
            let new_uids: Vec<u32> = server_uids
                .into_iter()
                .filter(|uid| !existing_uids.contains(uid))
                .collect();

            for uid in new_uids {
                let parsed =
                    match imap_password::fetch_message(&imap_cfg, &creds, &fi.imap_path, uid)
                        .await
                    {
                        Ok(p) => p,
                        Err(e) => {
                            warn!("fetch_message uid={uid} folder={}: {e}", fi.imap_path);
                            continue;
                        }
                    };

                messages_fetched += 1;

                let to_addrs =
                    serde_json::to_string(&parsed.to).unwrap_or_else(|_| "[]".to_string());
                let cc_addrs =
                    serde_json::to_string(&parsed.cc).unwrap_or_else(|_| "[]".to_string());

                let msg = Message {
                    id: None,
                    account_id,
                    folder_id,
                    uid: uid as i64,
                    message_id_header: parsed.message_id,
                    thread_id: None,
                    from_addr: parsed.from_addr,
                    from_name: parsed.from_name,
                    to_addrs: Some(to_addrs),
                    cc_addrs: Some(cc_addrs),
                    subject: parsed.subject,
                    date: parsed.date,
                    snippet: Some(parsed.snippet),
                    body_text: parsed.body_text,
                    body_html: parsed.body_html,
                    flags: Some("[]".to_string()),
                    labels: Some("[]".to_string()),
                    detected_lang: None,
                    lang_confidence: None,
                    has_attachments: Some(if parsed.has_attachments { 1 } else { 0 }),
                    size: Some(parsed.size as i64),
                    created_at: now(),
                };

                let new_id = queries::insert_message(&self.pool, &msg).await?;

                if new_id > 0 {
                    messages_new += 1;
                    // Emit email:new event.
                    let _ = self.app.emit(
                        "email:new",
                        serde_json::json!({
                            "message_id": new_id,
                            "account_id": account_id,
                            "folder_id": folder_id,
                            "uid": uid,
                        }),
                    );
                }

                // Emit sync:progress every 5 messages.
                if messages_fetched % 5 == 0 {
                    let status = SyncStatus {
                        account_id,
                        account_email: email.clone(),
                        folder_count,
                        messages_fetched,
                        messages_new,
                        finished_at: None,
                        error: None,
                    };
                    let _ = self.app.emit("sync:progress", &status);
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        let final_status = SyncStatus {
            account_id,
            account_email: email.clone(),
            folder_count,
            messages_fetched,
            messages_new,
            finished_at: Some(now()),
            error: None,
        };

        // Cache final status.
        self.statuses
            .lock()
            .await
            .insert(account_id, final_status.clone());

        // Emit a final sync:progress with finished_at set.
        let _ = self.app.emit("sync:progress", &final_status);

        info!(
            "sync_account: done for {email} — {messages_new} new, {messages_fetched} fetched, {duration_ms}ms"
        );

        Ok(AccountSyncReport {
            account_id,
            folders: folder_names,
            new_messages: messages_new,
            duration_ms,
        })
    }

    /// Sync all accounts sequentially (active only).
    pub async fn sync_all(&self) -> Vec<AccountSyncReport> {
        let accounts = match queries::list_accounts(&self.pool).await {
            Ok(a) => a,
            Err(e) => {
                warn!("sync_all: list_accounts failed: {e}");
                return Vec::new();
            }
        };

        let mut reports = Vec::new();
        for account in accounts {
            // Skip inactive accounts.
            if account.status.as_deref() == Some("inactive") {
                continue;
            }
            let id = match account.id {
                Some(id) => id,
                None => continue,
            };
            match self.sync_account(id).await {
                Ok(report) => reports.push(report),
                Err(e) => {
                    warn!("sync_all: sync_account({id}) failed: {e}");
                    // Cache error status.
                    let email = account.email.unwrap_or_else(|| format!("account_{id}"));
                    let err_status = SyncStatus {
                        account_id: id,
                        account_email: email,
                        folder_count: 0,
                        messages_fetched: 0,
                        messages_new: 0,
                        finished_at: Some(now()),
                        error: Some(e.to_string()),
                    };
                    self.statuses.lock().await.insert(id, err_status);
                }
            }
        }
        reports
    }

    /// Background polling loop. Spawn via `tokio::spawn(engine.run_polling_loop())`.
    /// Never panics; errors are logged and the loop continues.
    ///
    /// TODO: Replace with IMAP IDLE once the IDLE command is implemented in
    /// `imap_password.rs`. For now this polls every `POLL_INTERVAL_SECS` seconds.
    pub async fn run_polling_loop(self: Arc<Self>) {
        info!("sync polling loop started (interval={POLL_INTERVAL_SECS}s)");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
            info!("sync polling loop: triggering sync_all");
            self.sync_all().await;
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns the current unix timestamp as i64 seconds.
fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Resolve an `ImapConfig` from the account row, falling back to the provider
/// catalog if the account's imap_host is empty.
fn resolve_imap_config(account: &Account) -> AppResult<ImapConfig> {
    // If the account row has an explicit imap_host, use it.
    if let Some(host) = account
        .imap_host
        .as_deref()
        .filter(|h| !h.is_empty())
    {
        let port = account.imap_port.unwrap_or(993) as u16;
        let encryption = parse_encryption(account.imap_encryption.as_deref());
        return Ok(ImapConfig {
            host: host.to_string(),
            port,
            encryption,
        });
    }

    // Fall back to provider catalog.
    let provider_str = account
        .provider
        .as_deref()
        .unwrap_or("generic_imap");

    let kind: ProviderKind = provider_str
        .parse()
        .unwrap_or(ProviderKind::GenericImap);

    // Convert storage::models::ProviderKind -> accounts::provider::ProviderKind
    use crate::accounts::provider::ProviderKind as AccProvider;
    let acc_kind = match kind {
        ProviderKind::Gmail => AccProvider::Gmail,
        ProviderKind::Outlook => AccProvider::Outlook,
        ProviderKind::ICloud => AccProvider::ICloud,
        ProviderKind::Qq => AccProvider::Qq,
        ProviderKind::Mail163 => AccProvider::Mail163,
        ProviderKind::GenericImap => AccProvider::GenericImap,
    };
    let cfg = provider_catalog::config(acc_kind);
    cfg.imap.ok_or_else(|| {
        AppError::Provider(format!(
            "provider '{provider_str}' has no IMAP config; \
             please provide imap_host/port/encryption explicitly"
        ))
    })
}

/// Parse an encryption string from the DB ("tls", "starttls", "none").
fn parse_encryption(s: Option<&str>) -> Encryption {
    match s.map(|v| v.to_lowercase()).as_deref() {
        Some("starttls") => Encryption::StartTls,
        Some("none") => Encryption::None,
        _ => Encryption::Tls,
    }
}

/// Map IMAP special-use attributes (e.g. `\\Inbox`) to a `FolderKind`.
/// The `attributes` vec contains debug-formatted strings like `"Extension(\"\\\\Inbox\")"`.
fn infer_folder_kind(attributes: &[String]) -> FolderKind {
    let joined = attributes.join(" ").to_lowercase();
    if joined.contains("inbox") {
        FolderKind::Inbox
    } else if joined.contains("sent") {
        FolderKind::Sent
    } else if joined.contains("draft") {
        FolderKind::Drafts
    } else if joined.contains("trash") || joined.contains("deleted") {
        FolderKind::Trash
    } else if joined.contains("spam") || joined.contains("junk") {
        FolderKind::Spam
    } else if joined.contains("archive") || joined.contains("all") {
        FolderKind::Archive
    } else {
        FolderKind::Custom
    }
}

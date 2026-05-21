use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::accounts::imap_password::{self, ImapAuth, ImapCredentials};
use crate::accounts::oauth::store as oauth_store;
use crate::accounts::oauth::OAuthFlow;
use crate::accounts::provider::{Encryption, ImapConfig, ProviderKind as AccProviderKind};
use crate::accounts::provider_catalog;
use crate::error::{AppError, AppResult};
use crate::storage::models::{Account, Attachment, Folder, FolderKind, Message, ProviderKind};
use crate::storage::queries;

pub const MAX_PER_FOLDER: u32 = 50;
pub const POLL_INTERVAL_SECS: u64 = 300; // 5 min; IDLE handles real-time INBOX
pub const IDLE_RESTART_BACKOFF_INITIAL_SECS: u64 = 15;
pub const IDLE_RESTART_BACKOFF_MAX_SECS: u64 = 300;

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
    oauth: Arc<OAuthFlow>,
    /// Last reported status per account_id.
    statuses: Arc<Mutex<HashMap<i64, SyncStatus>>>,
}

impl SyncEngine {
    pub fn new(pool: SqlitePool, app: AppHandle, oauth: Arc<OAuthFlow>) -> Self {
        Self {
            pool,
            app,
            oauth,
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

        // 3. Build auth (XOAUTH2 for OAuth providers with tokens, password otherwise).
        let auth = self.build_auth(&account, &email).await?;

        // 4. Fetch folder list and upsert each into `folders` table.
        let folder_infos = imap_password::fetch_folders(&imap_cfg, &auth).await?;

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

            // SELECT folder + capture UIDVALIDITY/UIDNEXT, then UID SEARCH.
            let state = match imap_password::fetch_folder_state(
                &imap_cfg,
                &auth,
                &fi.imap_path,
                MAX_PER_FOLDER,
            )
            .await
            {
                Ok(s) => s,
                Err(e) => {
                    warn!("fetch_folder_state failed for {}: {e}", fi.imap_path);
                    continue;
                }
            };

            // UIDVALIDITY check (RFC 3501 §2.3.1.1).
            // If the server-reported validity differs from what we cached, the
            // server has re-numbered messages and every UID we hold is stale.
            let stored_validity = queries::get_folder_uid_validity(&self.pool, folder_id)
                .await
                .unwrap_or(None);
            let server_validity_i64 = state.uid_validity as i64;
            if let Some(stored) = stored_validity {
                if stored != server_validity_i64 {
                    warn!(
                        folder = %fi.imap_path,
                        stored,
                        server = server_validity_i64,
                        "UIDVALIDITY changed — wiping local cache for folder"
                    );
                    let _ = queries::delete_messages_in_folder(&self.pool, folder_id).await;
                }
            }
            // Persist current validity + uid_next.
            let _ = queries::update_folder_uid_validity(
                &self.pool,
                folder_id,
                server_validity_i64,
                state.uid_next as i64,
            )
            .await;

            let server_uids = state.recent_uids;

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
                    match imap_password::fetch_message(&imap_cfg, &auth, &fi.imap_path, uid)
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

                    // Persist attachments to local cache + insert rows.
                    if !parsed.attachments.is_empty() {
                        if let Err(e) =
                            store_attachments(&self.app, &self.pool, new_id, &parsed.attachments)
                                .await
                        {
                            warn!("store_attachments failed for msg {new_id}: {e}");
                        }
                    }

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

    /// Background polling loop. Runs alongside IDLE workers and covers folders
    /// other than INBOX (sent, archive, etc.) plus serves as a fallback for
    /// servers that do not honor IDLE.
    pub async fn run_polling_loop(self: Arc<Self>) {
        info!("sync polling loop started (interval={POLL_INTERVAL_SECS}s)");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
            info!("sync polling loop: triggering sync_all");
            self.sync_all().await;
        }
    }

    /// Spawn one IMAP IDLE worker per active account on INBOX. Each worker
    /// holds a long-lived connection and triggers `sync_account` whenever the
    /// server reports EXISTS / RECENT / EXPUNGE. On the 29-minute keepalive
    /// the IDLE is restarted; on errors there's exponential backoff.
    pub async fn spawn_idle_workers(self: Arc<Self>) {
        let accounts = match queries::list_accounts(&self.pool).await {
            Ok(a) => a,
            Err(e) => {
                warn!("spawn_idle_workers: list_accounts failed: {e}");
                return;
            }
        };
        for account in accounts {
            if account.status.as_deref() == Some("inactive") {
                continue;
            }
            let Some(id) = account.id else { continue };
            let this = self.clone();
            tokio::spawn(async move {
                this.idle_loop_for_account(id).await;
            });
        }
    }

    /// Long-running IDLE loop for a single account. Re-enters IDLE forever,
    /// with exponential backoff on errors.
    async fn idle_loop_for_account(self: Arc<Self>, account_id: i64) {
        let mut backoff = IDLE_RESTART_BACKOFF_INITIAL_SECS;
        loop {
            match self.run_idle_once(account_id).await {
                Ok(notified) => {
                    backoff = IDLE_RESTART_BACKOFF_INITIAL_SECS;
                    if notified {
                        info!("idle({account_id}): server reported change, syncing");
                        if let Err(e) = self.sync_account(account_id).await {
                            warn!("idle({account_id}): post-notify sync failed: {e}");
                        }
                    }
                }
                Err(e) => {
                    warn!("idle({account_id}): {e}; retrying in {backoff}s");
                    tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
                    backoff = (backoff * 2).min(IDLE_RESTART_BACKOFF_MAX_SECS);
                }
            }
        }
    }

    /// One IDLE pass on INBOX. Returns Ok(true) when the server pushed a
    /// notification, Ok(false) on the 29-minute timeout.
    async fn run_idle_once(&self, account_id: i64) -> AppResult<bool> {
        let account = queries::get_account(self.pool.clone(), account_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("account {account_id} not found")))?;
        let email = account
            .email
            .clone()
            .unwrap_or_else(|| format!("account_{account_id}"));

        let imap_cfg = resolve_imap_config(&account)?;
        let auth = self.build_auth(&account, &email).await?;

        let event = imap_password::idle_wait_once(&imap_cfg, &auth, "INBOX").await?;
        Ok(matches!(event, imap_password::IdleEvent::Notified))
    }

    /// Build an `ImapAuth` for an account: XOAUTH2 for OAuth providers with
    /// stored tokens, password (LOGIN) otherwise.
    async fn build_auth(&self, account: &Account, email: &str) -> AppResult<ImapAuth> {
        let provider_str = account.provider.as_deref().unwrap_or("generic_imap");
        let acc_kind: AccProviderKind = match provider_str {
            "gmail" => AccProviderKind::Gmail,
            "outlook" => AccProviderKind::Outlook,
            "icloud" => AccProviderKind::ICloud,
            "qq" => AccProviderKind::Qq,
            "mail163" => AccProviderKind::Mail163,
            _ => AccProviderKind::GenericImap,
        };

        let use_oauth = matches!(
            acc_kind,
            AccProviderKind::Gmail | AccProviderKind::Outlook
        ) && oauth_store::has_tokens(acc_kind, email).unwrap_or(false);

        if use_oauth {
            let tokens =
                oauth_store::get_valid_tokens(&self.oauth, acc_kind, email, 300).await?;
            Ok(ImapAuth::XOAuth2 {
                email: email.to_string(),
                access_token: tokens.access_token,
            })
        } else {
            let password = super::credentials::load_imap_password(email)?.ok_or_else(|| {
                AppError::Auth(format!(
                    "no IMAP password or OAuth tokens stored for {email}"
                ))
            })?;
            Ok(ImapAuth::Password(ImapCredentials {
                username: email.to_string(),
                password,
            }))
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Sanitize a filename so it is safe to write to local disk.
fn safe_filename(name: &str, fallback_idx: usize) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ' ' | '(' | ')') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches(|c: char| c == '.' || c.is_whitespace());
    if trimmed.is_empty() {
        format!("attachment-{fallback_idx}.bin")
    } else {
        trimmed.to_string()
    }
}

/// Resolve `<app_data_dir>/attachments/<message_id>` and create it on disk.
async fn attachments_dir(
    app: &AppHandle,
    message_id: i64,
) -> AppResult<std::path::PathBuf> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Other(e.to_string()))?
        .join("attachments")
        .join(message_id.to_string());
    tokio::fs::create_dir_all(&base).await?;
    Ok(base)
}

/// Write each parsed attachment to disk and insert a row into the
/// `attachments` table. Attachments without unique filenames are de-duplicated
/// by appending an index suffix.
async fn store_attachments(
    app: &AppHandle,
    pool: &SqlitePool,
    message_id: i64,
    parsed: &[crate::accounts::provider::ParsedAttachment],
) -> AppResult<()> {
    let dir = attachments_dir(app, message_id).await?;
    let mut used = std::collections::HashSet::<String>::new();

    for (idx, att) in parsed.iter().enumerate() {
        let mut name = safe_filename(&att.filename, idx);
        if used.contains(&name) {
            let (stem, ext) = match name.rsplit_once('.') {
                Some((s, e)) => (s.to_string(), format!(".{e}")),
                None => (name.clone(), String::new()),
            };
            name = format!("{stem}-{idx}{ext}");
        }
        used.insert(name.clone());

        let path = dir.join(&name);
        tokio::fs::write(&path, &att.data).await?;

        let row = Attachment {
            id: None,
            message_id,
            filename: Some(att.filename.clone()),
            mime: att.mime.clone(),
            size: Some(att.data.len() as i64),
            content_id: att.content_id.clone(),
            blob_path: Some(path.to_string_lossy().to_string()),
            downloaded: Some(1),
        };
        queries::insert_attachment(pool, &row).await?;
    }
    Ok(())
}

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

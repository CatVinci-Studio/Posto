use std::collections::HashSet;

use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use super::models::{
    Account, AgentRun, Attachment, Folder, Memory, MemoryType, Message, Task, Translation,
};

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

pub async fn list_accounts(pool: &SqlitePool) -> AppResult<Vec<Account>> {
    let rows = sqlx::query_as::<_, Account>(
        "SELECT id, provider, email, display_name, oauth_token_ref,
                imap_host, imap_port, imap_encryption,
                smtp_host, smtp_port, smtp_encryption,
                status, created_at, updated_at
         FROM accounts
         ORDER BY id ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Insert a new account row.  If `account.created_at` or `updated_at` are 0
/// they are replaced with the current unix timestamp.
pub async fn insert_account(pool: &SqlitePool, account: &Account) -> AppResult<i64> {
    let now = chrono::Utc::now().timestamp();
    let created_at = if account.created_at == 0 { now } else { account.created_at };
    let updated_at = if account.updated_at == 0 { now } else { account.updated_at };

    let result = sqlx::query(
        "INSERT INTO accounts
            (provider, email, display_name, oauth_token_ref,
             imap_host, imap_port, imap_encryption,
             smtp_host, smtp_port, smtp_encryption,
             status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
    )
    .bind(&account.provider)
    .bind(&account.email)
    .bind(&account.display_name)
    .bind(&account.oauth_token_ref)
    .bind(&account.imap_host)
    .bind(account.imap_port)
    .bind(&account.imap_encryption)
    .bind(&account.smtp_host)
    .bind(account.smtp_port)
    .bind(&account.smtp_encryption)
    .bind(account.status.as_deref().unwrap_or("active"))
    .bind(created_at)
    .bind(updated_at)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn get_account_by_email(pool: &SqlitePool, email: &str) -> AppResult<Option<Account>> {
    let row = sqlx::query_as::<_, Account>(
        "SELECT id, provider, email, display_name, oauth_token_ref,
                imap_host, imap_port, imap_encryption,
                smtp_host, smtp_port, smtp_encryption,
                status, created_at, updated_at
         FROM accounts
         WHERE email = ?1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

// ---------------------------------------------------------------------------
// Folders
// ---------------------------------------------------------------------------

/// Insert or update a folder row (keyed on account_id + imap_path).
pub async fn upsert_folder(pool: &SqlitePool, folder: &Folder) -> AppResult<i64> {
    let result = sqlx::query(
        "INSERT INTO folders (account_id, name, kind, imap_path, uid_validity, uid_next)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(account_id, imap_path) DO UPDATE SET
             name         = excluded.name,
             kind         = excluded.kind,
             uid_validity = excluded.uid_validity,
             uid_next     = excluded.uid_next",
    )
    .bind(folder.account_id)
    .bind(&folder.name)
    .bind(&folder.kind)
    .bind(&folder.imap_path)
    .bind(folder.uid_validity)
    .bind(folder.uid_next)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

/// Insert a message row.  Returns the new rowid, or 0 on a UNIQUE conflict
/// (the row already exists — silently ignored).
pub async fn insert_message(pool: &SqlitePool, msg: &Message) -> AppResult<i64> {
    let now = chrono::Utc::now().timestamp();
    let created_at = if msg.created_at == 0 { now } else { msg.created_at };

    let result = sqlx::query(
        "INSERT OR IGNORE INTO messages
            (account_id, folder_id, uid, message_id_header, thread_id,
             from_addr, from_name, to_addrs, cc_addrs, subject, date,
             snippet, body_text, body_html, flags, labels,
             detected_lang, lang_confidence, has_attachments, size, created_at)
         VALUES
            (?1,  ?2,  ?3,  ?4,  ?5,
             ?6,  ?7,  ?8,  ?9,  ?10, ?11,
             ?12, ?13, ?14, ?15, ?16,
             ?17, ?18, ?19, ?20, ?21)",
    )
    .bind(msg.account_id)
    .bind(msg.folder_id)
    .bind(msg.uid)
    .bind(&msg.message_id_header)
    .bind(&msg.thread_id)
    .bind(&msg.from_addr)
    .bind(&msg.from_name)
    .bind(&msg.to_addrs)
    .bind(&msg.cc_addrs)
    .bind(&msg.subject)
    .bind(msg.date)
    .bind(&msg.snippet)
    .bind(&msg.body_text)
    .bind(&msg.body_html)
    .bind(&msg.flags)
    .bind(&msg.labels)
    .bind(&msg.detected_lang)
    .bind(msg.lang_confidence)
    .bind(msg.has_attachments.unwrap_or(0))
    .bind(msg.size)
    .bind(created_at)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn list_messages_for_folder(
    pool: &SqlitePool,
    folder_id: i64,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<Message>> {
    let rows = sqlx::query_as::<_, Message>(
        "SELECT id, account_id, folder_id, uid, message_id_header, thread_id,
                from_addr, from_name, to_addrs, cc_addrs, subject, date,
                snippet, body_text, body_html, flags, labels,
                detected_lang, lang_confidence, has_attachments, size, created_at
         FROM messages
         WHERE folder_id = ?1
         ORDER BY date DESC
         LIMIT ?2 OFFSET ?3",
    )
    .bind(folder_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ---------------------------------------------------------------------------
// Additional account / folder / message helpers (added by sync module)
// ---------------------------------------------------------------------------

/// Fetch a single account by its primary-key id.
pub async fn get_account(pool: SqlitePool, id: i64) -> AppResult<Option<Account>> {
    let row = sqlx::query_as::<_, Account>(
        "SELECT id, provider, email, display_name, oauth_token_ref,
                imap_host, imap_port, imap_encryption,
                smtp_host, smtp_port, smtp_encryption,
                status, created_at, updated_at
         FROM accounts
         WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?;
    Ok(row)
}

/// Return the `id` of a folder identified by `(account_id, imap_path)`, if any.
pub async fn get_folder_id_by_path(
    pool: SqlitePool,
    account_id: i64,
    imap_path: &str,
) -> AppResult<Option<i64>> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM folders WHERE account_id = ?1 AND imap_path = ?2",
    )
    .bind(account_id)
    .bind(imap_path)
    .fetch_optional(&pool)
    .await?;
    Ok(row.map(|(id,)| id))
}

/// Return all folders stored for a given account.
pub async fn list_folders(pool: SqlitePool, account_id: i64) -> AppResult<Vec<Folder>> {
    let rows = sqlx::query_as::<_, Folder>(
        "SELECT id, account_id, name, kind, imap_path, uid_validity, uid_next
         FROM folders
         WHERE account_id = ?1
         ORDER BY id ASC",
    )
    .bind(account_id)
    .fetch_all(&pool)
    .await?;
    Ok(rows)
}

/// Read the stored UIDVALIDITY for a folder.
pub async fn get_folder_uid_validity(
    pool: &SqlitePool,
    folder_id: i64,
) -> AppResult<Option<i64>> {
    let row: Option<(Option<i64>,)> = sqlx::query_as(
        "SELECT uid_validity FROM folders WHERE id = ?1",
    )
    .bind(folder_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|(v,)| v))
}

/// Persist new UIDVALIDITY / UIDNEXT after a successful SELECT.
pub async fn update_folder_uid_validity(
    pool: &SqlitePool,
    folder_id: i64,
    uid_validity: i64,
    uid_next: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE folders
         SET uid_validity = ?1, uid_next = ?2
         WHERE id = ?3",
    )
    .bind(uid_validity)
    .bind(uid_next)
    .bind(folder_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Wipe all messages (and FTS / agent_runs cascades) in a folder. Called when
/// the server reports a UIDVALIDITY change — previously-cached UIDs are no
/// longer meaningful and must be re-fetched.
pub async fn delete_messages_in_folder(
    pool: &SqlitePool,
    folder_id: i64,
) -> AppResult<u64> {
    let result = sqlx::query("DELETE FROM messages WHERE folder_id = ?1")
        .bind(folder_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Return the set of UIDs already stored for a given `folder_id`.
pub async fn list_existing_uids(pool: SqlitePool, folder_id: i64) -> AppResult<HashSet<u32>> {
    let rows: Vec<(i64,)> = sqlx::query_as(
        "SELECT uid FROM messages WHERE folder_id = ?1",
    )
    .bind(folder_id)
    .fetch_all(&pool)
    .await?;
    Ok(rows.into_iter().map(|(uid,)| uid as u32).collect())
}

// ---------------------------------------------------------------------------
// Attachments
// ---------------------------------------------------------------------------

pub async fn insert_attachment(pool: &SqlitePool, att: &Attachment) -> AppResult<i64> {
    let result = sqlx::query(
        "INSERT INTO attachments
            (message_id, filename, mime, size, content_id, blob_path, downloaded)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(att.message_id)
    .bind(&att.filename)
    .bind(&att.mime)
    .bind(att.size)
    .bind(&att.content_id)
    .bind(&att.blob_path)
    .bind(att.downloaded.unwrap_or(0))
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn list_attachments(
    pool: &SqlitePool,
    message_id: i64,
) -> AppResult<Vec<Attachment>> {
    let rows = sqlx::query_as::<_, Attachment>(
        "SELECT id, message_id, filename, mime, size, content_id, blob_path, downloaded
         FROM attachments
         WHERE message_id = ?1
         ORDER BY id ASC",
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_attachment(pool: &SqlitePool, id: i64) -> AppResult<Attachment> {
    sqlx::query_as::<_, Attachment>(
        "SELECT id, message_id, filename, mime, size, content_id, blob_path, downloaded
         FROM attachments
         WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("attachment id={id}")))
}

// ---------------------------------------------------------------------------
// Memories
// ---------------------------------------------------------------------------

pub async fn insert_memory(pool: &SqlitePool, memory: &Memory) -> AppResult<i64> {
    let now = chrono::Utc::now().timestamp();
    let created_at = if memory.created_at == 0 { now } else { memory.created_at };

    let result = sqlx::query(
        "INSERT INTO memories
            (type, scope, key, content, embedding, importance, pinned,
             source_message_id, created_at, last_used_at, use_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
    )
    .bind(&memory.memory_type)
    .bind(&memory.scope)
    .bind(&memory.key)
    .bind(&memory.content)
    .bind(&memory.embedding)
    .bind(memory.importance.unwrap_or(0.5))
    .bind(memory.pinned.unwrap_or(0))
    .bind(memory.source_message_id)
    .bind(created_at)
    .bind(memory.last_used_at)
    .bind(memory.use_count.unwrap_or(0))
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Search memories by scope prefix and optional type filter.
/// `scope_prefix` is matched with LIKE '<prefix>%'.
pub async fn search_memories(
    pool: &SqlitePool,
    scope_prefix: &str,
    mtype: Option<MemoryType>,
    limit: usize,
) -> AppResult<Vec<Memory>> {
    let like_pattern = format!("{scope_prefix}%");
    let limit_i64 = limit as i64;

    let rows = match mtype {
        Some(t) => {
            sqlx::query_as::<_, Memory>(
                "SELECT id, type, scope, key, content, embedding, importance, pinned,
                        source_message_id, created_at, last_used_at, use_count
                 FROM memories
                 WHERE scope LIKE ?1
                   AND type = ?2
                 ORDER BY importance DESC, last_used_at DESC
                 LIMIT ?3",
            )
            .bind(&like_pattern)
            .bind(t.as_str())
            .bind(limit_i64)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, Memory>(
                "SELECT id, type, scope, key, content, embedding, importance, pinned,
                        source_message_id, created_at, last_used_at, use_count
                 FROM memories
                 WHERE scope LIKE ?1
                 ORDER BY importance DESC, last_used_at DESC
                 LIMIT ?2",
            )
            .bind(&like_pattern)
            .bind(limit_i64)
            .fetch_all(pool)
            .await?
        }
    };

    Ok(rows)
}

// ---------------------------------------------------------------------------
// Messages — additional helpers (new, additive)
// ---------------------------------------------------------------------------

/// Load a single message by primary-key id.
pub async fn get_message(pool: &SqlitePool, id: i64) -> AppResult<Message> {
    let row = sqlx::query_as::<_, Message>(
        "SELECT id, account_id, folder_id, uid, message_id_header, thread_id,
                from_addr, from_name, to_addrs, cc_addrs, subject, date,
                snippet, body_text, body_html, flags, labels,
                detected_lang, lang_confidence, has_attachments, size, created_at
         FROM messages
         WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("message id={id}")))?;
    Ok(row)
}

/// Update detected_lang for a message if it is currently NULL.
pub async fn update_message_lang(pool: &SqlitePool, id: i64, lang: &str) -> AppResult<()> {
    sqlx::query(
        "UPDATE messages SET detected_lang = ?1
         WHERE id = ?2 AND detected_lang IS NULL",
    )
    .bind(lang)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// AgentRun — new helpers
// ---------------------------------------------------------------------------

pub async fn insert_agent_run(pool: &SqlitePool, run: &AgentRun) -> AppResult<i64> {
    let now = chrono::Utc::now().timestamp();
    let created_at = if run.created_at == 0 { now } else { run.created_at };

    let result = sqlx::query(
        "INSERT INTO agent_runs
            (message_id, agent_type, input_summary, output,
             tokens_in, tokens_out, model, cost, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )
    .bind(run.message_id)
    .bind(&run.agent_type)
    .bind(&run.input_summary)
    .bind(&run.output)
    .bind(run.tokens_in)
    .bind(run.tokens_out)
    .bind(&run.model)
    .bind(run.cost)
    .bind(created_at)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn list_agent_runs(
    pool: &SqlitePool,
    message_id: Option<i64>,
    limit: i64,
) -> AppResult<Vec<AgentRun>> {
    let rows = match message_id {
        Some(mid) => {
            sqlx::query_as::<_, AgentRun>(
                "SELECT id, message_id, agent_type, input_summary, output,
                        tokens_in, tokens_out, model, cost, created_at
                 FROM agent_runs
                 WHERE message_id = ?1
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .bind(mid)
            .bind(limit)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, AgentRun>(
                "SELECT id, message_id, agent_type, input_summary, output,
                        tokens_in, tokens_out, model, cost, created_at
                 FROM agent_runs
                 ORDER BY created_at DESC
                 LIMIT ?1",
            )
            .bind(limit)
            .fetch_all(pool)
            .await?
        }
    };
    Ok(rows)
}

// ---------------------------------------------------------------------------
// Tasks — new helpers
// ---------------------------------------------------------------------------

pub async fn insert_task(pool: &SqlitePool, task: &Task) -> AppResult<i64> {
    let now = chrono::Utc::now().timestamp();
    let created_at = if task.created_at == 0 { now } else { task.created_at };

    let result = sqlx::query(
        "INSERT INTO tasks
            (message_id, account_id, title, due_at, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(task.message_id)
    .bind(task.account_id)
    .bind(&task.title)
    .bind(task.due_at)
    .bind(task.status.as_deref().unwrap_or("open"))
    .bind(created_at)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

// ---------------------------------------------------------------------------
// Memories — additional helpers
// ---------------------------------------------------------------------------

/// Load candidate memories for semantic retrieval: scoped, embedding-present,
/// ranked by pinned+importance+recency. Always-load pinned ones plus the top
/// `limit` non-pinned candidates from the requested scopes.
pub async fn load_memories_by_scopes(
    pool: &SqlitePool,
    scopes: &[String],
    limit: i64,
) -> AppResult<Vec<Memory>> {
    if scopes.is_empty() {
        return Ok(vec![]);
    }

    // Build a parameterised IN clause dynamically.
    let placeholders: String = scopes
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect::<Vec<_>>()
        .join(", ");

    // Pre-filter: scope match AND has embedding bytes. Pinned rows sort first
    // so retrieval guarantees they always survive truncation.
    let sql = format!(
        "SELECT id, type, scope, key, content, embedding, importance, pinned,
                source_message_id, created_at, last_used_at, use_count
         FROM memories
         WHERE scope IN ({placeholders})
           AND embedding IS NOT NULL
           AND length(embedding) > 0
         ORDER BY pinned DESC,
                  importance DESC,
                  COALESCE(last_used_at, created_at) DESC
         LIMIT ?{}",
        scopes.len() + 1
    );

    let mut query = sqlx::query_as::<_, Memory>(&sql);
    for s in scopes {
        query = query.bind(s.as_str());
    }
    query = query.bind(limit);

    let rows = query.fetch_all(pool).await?;
    Ok(rows)
}

/// Update importance on an existing memory row.
pub async fn update_memory_importance(
    pool: &SqlitePool,
    id: i64,
    importance: f64,
) -> AppResult<()> {
    sqlx::query("UPDATE memories SET importance = ?1 WHERE id = ?2")
        .bind(importance)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Update pinned flag on an existing memory.
pub async fn update_memory_pinned(pool: &SqlitePool, id: i64, pinned: bool) -> AppResult<()> {
    sqlx::query("UPDATE memories SET pinned = ?1 WHERE id = ?2")
        .bind(if pinned { 1i64 } else { 0i64 })
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Delete a memory by id.
pub async fn delete_memory_by_id(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM memories WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Touch last_used_at and increment use_count.
pub async fn touch_memory(pool: &SqlitePool, id: i64) -> AppResult<()> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "UPDATE memories
         SET last_used_at = ?1,
             use_count    = COALESCE(use_count, 0) + 1
         WHERE id = ?2",
    )
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Translations — new helpers
// ---------------------------------------------------------------------------

/// Return cached translated body_text for (message_id, target_lang), if any.
pub async fn get_translation(
    pool: &SqlitePool,
    msg_id: i64,
    lang: &str,
) -> AppResult<Option<String>> {
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT body_text FROM translations
         WHERE message_id = ?1 AND target_lang = ?2",
    )
    .bind(msg_id)
    .bind(lang)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|(t,)| t))
}

/// Insert a new translation row (or ignore if already exists).
pub async fn insert_translation(pool: &SqlitePool, t: &Translation) -> AppResult<()> {
    let now = chrono::Utc::now().timestamp();
    let created_at = if t.created_at == 0 { now } else { t.created_at };

    sqlx::query(
        "INSERT OR IGNORE INTO translations
            (message_id, target_lang, body_text, body_html, model, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(t.message_id)
    .bind(&t.target_lang)
    .bind(&t.body_text)
    .bind(&t.body_html)
    .bind(&t.model)
    .bind(created_at)
    .execute(pool)
    .await?;
    Ok(())
}

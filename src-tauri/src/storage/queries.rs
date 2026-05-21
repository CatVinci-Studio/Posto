use sqlx::SqlitePool;

use crate::error::AppResult;
use super::models::{Account, Folder, Memory, MemoryType, Message};

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

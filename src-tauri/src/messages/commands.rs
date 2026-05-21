use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, State};

use crate::storage::models::Message;

// ---------------------------------------------------------------------------
// Output shapes
// ---------------------------------------------------------------------------

/// Lightweight enrichment extracted from the latest `agent_runs` row of
/// type "summary" or "triage". Fields are best-effort: missing JSON keys
/// degrade gracefully to None.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageEnrichment {
    pub message_id: i64,
    pub category: Option<String>,
    pub priority: Option<f32>,
    pub summary: Option<String>,
    pub facts: Option<Vec<String>>,
    pub is_actionable: Option<bool>,
    pub needs_response: Option<bool>,
    pub task_count: Option<u32>,
    pub tasks: Option<Vec<SuggestedTask>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedTask {
    pub title: String,
    pub due_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxItem {
    pub message: Message,
    pub enrichment: Option<MessageEnrichment>,
}

// ---------------------------------------------------------------------------
// list_inbox_items
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct InboxFilter {
    pub account_id: Option<i64>,
    pub category: Option<String>,
    pub status: Option<String>,        // "today" | "week" | "newsletters" | "done"
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[tauri::command]
pub async fn list_inbox_items(
    filter: Option<InboxFilter>,
    pool: State<'_, SqlitePool>,
) -> Result<Vec<InboxItem>, String> {
    let filter = filter.unwrap_or_default();
    let limit = filter.limit.unwrap_or(200).clamp(1, 1000);
    let offset = filter.offset.unwrap_or(0);

    // Build dynamic WHERE clause.
    let mut where_parts: Vec<String> = Vec::new();
    let mut binds: Vec<(usize, BindValue)> = Vec::new();
    let mut idx = 1usize;

    if let Some(account_id) = filter.account_id {
        where_parts.push(format!("m.account_id = ?{idx}"));
        binds.push((idx, BindValue::I64(account_id)));
        idx += 1;
    }
    if let Some(status) = filter.status.as_deref() {
        match status {
            "today" => {
                let day_start = day_start_unix();
                where_parts.push(format!("m.date >= ?{idx}"));
                binds.push((idx, BindValue::I64(day_start)));
                idx += 1;
            }
            "week" => {
                let week_start = day_start_unix() - 6 * 86400;
                where_parts.push(format!("m.date >= ?{idx}"));
                binds.push((idx, BindValue::I64(week_start)));
                idx += 1;
            }
            "done" => {
                // Items archived locally.
                where_parts.push("(m.flags LIKE '%\"Archive\"%')".to_string());
            }
            "newsletters" => {
                // Category-tagged in agent_runs JSON.
                where_parts.push(
                    "EXISTS (SELECT 1 FROM agent_runs ar2 \
                     WHERE ar2.message_id = m.id \
                       AND ar2.output LIKE '%\"category\":\"newsletter\"%')"
                        .to_string(),
                );
            }
            _ => {}
        }
    }
    if let Some(cat) = filter.category.as_deref() {
        where_parts.push(format!(
            "EXISTS (SELECT 1 FROM agent_runs ar3 \
             WHERE ar3.message_id = m.id \
               AND ar3.output LIKE '%\"category\":\"{}\"%')",
            cat.replace('"', "")
        ));
    }

    // Always-on filter: hide locally-deleted rows.
    where_parts.push("(m.flags IS NULL OR m.flags NOT LIKE '%\"\\\\Deleted\"%')".to_string());

    let where_clause = if where_parts.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_parts.join(" AND "))
    };

    let limit_ph = idx;
    let offset_ph = idx + 1;

    let sql = format!(
        "SELECT m.id, m.account_id, m.folder_id, m.uid, m.message_id_header, m.thread_id,
                m.from_addr, m.from_name, m.to_addrs, m.cc_addrs, m.subject, m.date,
                m.snippet, m.body_text, m.body_html, m.flags, m.labels,
                m.detected_lang, m.lang_confidence, m.has_attachments, m.size, m.created_at,
                (
                  SELECT ar.output
                  FROM agent_runs ar
                  WHERE ar.message_id = m.id
                    AND ar.agent_type IN ('triage','summary','action','reflection')
                  ORDER BY CASE ar.agent_type
                             WHEN 'reflection' THEN 4
                             WHEN 'summary'    THEN 3
                             WHEN 'action'     THEN 2
                             WHEN 'triage'     THEN 1
                             ELSE 0
                           END DESC,
                           ar.created_at DESC
                  LIMIT 1
                ) AS enrichment_json
         FROM messages m
         {where_clause}
         ORDER BY m.date DESC
         LIMIT ?{limit_ph} OFFSET ?{offset_ph}",
    );

    let mut query = sqlx::query_as::<_, RowWithEnrichment>(&sql);
    for (_, b) in &binds {
        query = match b {
            BindValue::I64(v) => query.bind(*v),
        };
    }
    query = query.bind(limit).bind(offset);

    let rows = query.fetch_all(pool.inner()).await.map_err(|e| e.to_string())?;

    let items: Vec<InboxItem> = rows
        .into_iter()
        .map(|row| {
            let enrichment = row
                .enrichment_json
                .as_deref()
                .and_then(|raw| parse_enrichment(row.id.unwrap_or(0), raw));
            InboxItem {
                message: Message {
                    id: row.id,
                    account_id: row.account_id,
                    folder_id: row.folder_id,
                    uid: row.uid,
                    message_id_header: row.message_id_header,
                    thread_id: row.thread_id,
                    from_addr: row.from_addr,
                    from_name: row.from_name,
                    to_addrs: row.to_addrs,
                    cc_addrs: row.cc_addrs,
                    subject: row.subject,
                    date: row.date,
                    snippet: row.snippet,
                    body_text: row.body_text,
                    body_html: row.body_html,
                    flags: row.flags,
                    labels: row.labels,
                    detected_lang: row.detected_lang,
                    lang_confidence: row.lang_confidence,
                    has_attachments: row.has_attachments,
                    size: row.size,
                    created_at: row.created_at,
                },
                enrichment,
            }
        })
        .collect();

    Ok(items)
}

#[derive(sqlx::FromRow)]
struct RowWithEnrichment {
    id: Option<i64>,
    account_id: i64,
    folder_id: i64,
    uid: i64,
    message_id_header: Option<String>,
    thread_id: Option<String>,
    from_addr: Option<String>,
    from_name: Option<String>,
    to_addrs: Option<String>,
    cc_addrs: Option<String>,
    subject: Option<String>,
    date: Option<i64>,
    snippet: Option<String>,
    body_text: Option<String>,
    body_html: Option<String>,
    flags: Option<String>,
    labels: Option<String>,
    detected_lang: Option<String>,
    lang_confidence: Option<f64>,
    has_attachments: Option<i64>,
    size: Option<i64>,
    created_at: i64,
    enrichment_json: Option<String>,
}

enum BindValue {
    I64(i64),
}

fn parse_enrichment(message_id: i64, raw: &str) -> Option<MessageEnrichment> {
    let value: serde_json::Value = serde_json::from_str(raw).ok()?;
    Some(MessageEnrichment {
        message_id,
        category: value.get("category").and_then(|v| v.as_str()).map(String::from),
        priority: value.get("priority").and_then(|v| v.as_f64()).map(|f| f as f32),
        summary: value.get("summary").and_then(|v| v.as_str()).map(String::from),
        facts: value.get("facts").and_then(|v| v.as_array()).map(|arr| {
            arr.iter().filter_map(|x| x.as_str().map(String::from)).collect()
        }),
        is_actionable: value.get("is_actionable").and_then(|v| v.as_bool()),
        needs_response: value.get("needs_response").and_then(|v| v.as_bool()),
        task_count: value
            .get("task_count")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32)
            .or_else(|| {
                value
                    .get("tasks")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len() as u32)
            }),
        tasks: value.get("tasks").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|t| {
                    let title = t.get("title").and_then(|x| x.as_str())?;
                    let due_at = t.get("due_at").and_then(|x| x.as_i64());
                    Some(SuggestedTask {
                        title: title.to_string(),
                        due_at,
                    })
                })
                .collect()
        }),
    })
}

fn day_start_unix() -> i64 {
    let now = chrono::Local::now();
    now.date_naive()
        .and_hms_opt(0, 0, 0)
        .and_then(|nd| nd.and_local_timezone(chrono::Local).single())
        .map(|dt| dt.timestamp())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Flag mutation commands (local-only V1)
// ---------------------------------------------------------------------------

async fn read_flags(pool: &SqlitePool, message_id: i64) -> Result<Vec<String>, String> {
    let row: Option<(Option<String>,)> = sqlx::query_as("SELECT flags FROM messages WHERE id = ?1")
        .bind(message_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
    let raw = row
        .ok_or_else(|| format!("message {message_id} not found"))?
        .0
        .unwrap_or_else(|| "[]".to_string());
    let flags: Vec<String> = serde_json::from_str(&raw).unwrap_or_default();
    Ok(flags)
}

async fn write_flags(
    pool: &SqlitePool,
    message_id: i64,
    flags: &[String],
) -> Result<(), String> {
    let json = serde_json::to_string(flags).map_err(|e| e.to_string())?;
    sqlx::query("UPDATE messages SET flags = ?1 WHERE id = ?2")
        .bind(&json)
        .bind(message_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn mutate_flag(
    pool: &SqlitePool,
    message_id: i64,
    flag: &str,
    add: bool,
) -> Result<(), String> {
    let mut flags = read_flags(pool, message_id).await?;
    let exists = flags.iter().any(|f| f == flag);
    if add && !exists {
        flags.push(flag.to_string());
    } else if !add && exists {
        flags.retain(|f| f != flag);
    } else {
        return Ok(()); // no-op
    }
    write_flags(pool, message_id, &flags).await
}

/// Queue a server-side mirror operation. Picked up by a later flush worker.
async fn queue_pending_op(
    pool: &SqlitePool,
    account_id: Option<i64>,
    op_type: &str,
    payload: serde_json::Value,
) -> Result<(), String> {
    let json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO pending_ops (account_id, op_type, payload, status, retries, created_at)
         VALUES (?1, ?2, ?3, 'pending', 0, ?4)",
    )
    .bind(account_id)
    .bind(op_type)
    .bind(&json)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

async fn account_id_for_message(pool: &SqlitePool, message_id: i64) -> Option<i64> {
    let row: Option<(i64,)> = sqlx::query_as("SELECT account_id FROM messages WHERE id = ?1")
        .bind(message_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    row.map(|(id,)| id)
}

#[tauri::command]
pub async fn mark_read(
    message_id: i64,
    read: bool,
    pool: State<'_, SqlitePool>,
    app: AppHandle,
) -> Result<(), String> {
    mutate_flag(&pool, message_id, "\\Seen", read).await?;
    let account_id = account_id_for_message(&pool, message_id).await;
    queue_pending_op(
        &pool,
        account_id,
        "set_flag",
        serde_json::json!({"message_id": message_id, "flag": "\\Seen", "add": read}),
    )
    .await
    .ok();
    let _ = app.emit(
        "message:updated",
        serde_json::json!({"message_id": message_id, "flag": "Seen", "value": read}),
    );
    Ok(())
}

#[tauri::command]
pub async fn flag_message(
    message_id: i64,
    flagged: bool,
    pool: State<'_, SqlitePool>,
    app: AppHandle,
) -> Result<(), String> {
    mutate_flag(&pool, message_id, "\\Flagged", flagged).await?;
    let account_id = account_id_for_message(&pool, message_id).await;
    queue_pending_op(
        &pool,
        account_id,
        "set_flag",
        serde_json::json!({"message_id": message_id, "flag": "\\Flagged", "add": flagged}),
    )
    .await
    .ok();
    let _ = app.emit(
        "message:updated",
        serde_json::json!({"message_id": message_id, "flag": "Flagged", "value": flagged}),
    );
    Ok(())
}

#[tauri::command]
pub async fn archive_message(
    message_id: i64,
    pool: State<'_, SqlitePool>,
    app: AppHandle,
) -> Result<(), String> {
    mutate_flag(&pool, message_id, "Archive", true).await?;
    let account_id = account_id_for_message(&pool, message_id).await;
    queue_pending_op(
        &pool,
        account_id,
        "move_to_archive",
        serde_json::json!({"message_id": message_id}),
    )
    .await
    .ok();
    let _ = app.emit(
        "message:updated",
        serde_json::json!({"message_id": message_id, "flag": "Archive", "value": true}),
    );
    Ok(())
}

#[tauri::command]
pub async fn delete_message(
    message_id: i64,
    pool: State<'_, SqlitePool>,
    app: AppHandle,
) -> Result<(), String> {
    mutate_flag(&pool, message_id, "\\Deleted", true).await?;
    let account_id = account_id_for_message(&pool, message_id).await;
    queue_pending_op(
        &pool,
        account_id,
        "expunge",
        serde_json::json!({"message_id": message_id}),
    )
    .await
    .ok();
    let _ = app.emit(
        "message:updated",
        serde_json::json!({"message_id": message_id, "flag": "Deleted", "value": true}),
    );
    Ok(())
}

#[tauri::command]
pub async fn create_task_from_message(
    message_id: i64,
    title: Option<String>,
    pool: State<'_, SqlitePool>,
) -> Result<i64, String> {
    use crate::storage::models::Task;

    let row: Option<(Option<i64>, Option<String>)> =
        sqlx::query_as("SELECT account_id, subject FROM messages WHERE id = ?1")
            .bind(message_id)
            .fetch_optional(pool.inner())
            .await
            .map_err(|e| e.to_string())?;
    let (account_id, subject) =
        row.ok_or_else(|| format!("message {message_id} not found"))?;

    let final_title = title
        .or(subject)
        .unwrap_or_else(|| format!("Follow up on message {message_id}"));

    let task = Task {
        id: None,
        message_id: Some(message_id),
        account_id,
        title: Some(final_title),
        due_at: None,
        status: Some("open".to_string()),
        created_at: chrono::Utc::now().timestamp(),
    };

    crate::storage::queries::insert_task(&pool, &task)
        .await
        .map_err(|e| e.to_string())
}

use sqlx::SqlitePool;

use crate::llm::{self, LlmProvider, OpenAi};
use crate::storage::models::Memory;
use crate::storage::queries;

use super::retriever::{retrieve, RetrievedMemory};
use super::store::{delete, encode_embedding, pin, save};

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// List memories filtered by optional scope and/or type.
#[tauri::command]
pub async fn list_memories(
    scope: Option<String>,
    mtype: Option<String>,
    limit: Option<i64>,
    pool: tauri::State<'_, SqlitePool>,
) -> Result<Vec<Memory>, String> {
    let limit = limit.unwrap_or(100);
    let rows = if let Some(ref s) = scope {
        let mtype_enum = mtype
            .as_deref()
            .map(|t| t.parse().map_err(|e: String| e))
            .transpose()
            .map_err(|e| e)?;
        queries::search_memories(&pool, s, mtype_enum, limit as usize)
            .await
            .map_err(|e| e.to_string())?
    } else {
        // No scope filter — load from all scopes using a broad wildcard.
        let mtype_enum = mtype
            .as_deref()
            .map(|t| t.parse().map_err(|e: String| e))
            .transpose()
            .map_err(|e| e)?;
        queries::search_memories(&pool, "", mtype_enum, limit as usize)
            .await
            .map_err(|e| e.to_string())?
    };
    Ok(rows)
}

/// Toggle the pinned flag on a memory.
#[tauri::command]
pub async fn pin_memory(
    id: i64,
    pinned: bool,
    pool: tauri::State<'_, SqlitePool>,
) -> Result<(), String> {
    pin(&pool, id, pinned).await.map_err(|e| e.to_string())
}

/// Hard-delete a memory by id.
#[tauri::command]
pub async fn delete_memory(id: i64, pool: tauri::State<'_, SqlitePool>) -> Result<(), String> {
    delete(&pool, id).await.map_err(|e| e.to_string())
}

/// Manually add a memory (no LLM embedding — embedding is generated here).
#[tauri::command]
pub async fn add_memory_manual(
    r#type: String,
    scope: String,
    content: String,
    key: Option<String>,
    pool: tauri::State<'_, SqlitePool>,
) -> Result<i64, String> {
    let api_key = llm::secrets::load_openai_api_key()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no LLM provider configured".to_string())?;
    let llm_provider = OpenAi::with_api_key(api_key);

    let vecs = llm_provider
        .embed(None, vec![content.clone()])
        .await
        .map_err(|e| e.to_string())?;
    let emb_bytes = vecs
        .into_iter()
        .next()
        .map(|v| encode_embedding(&v))
        .unwrap_or_default();

    let now = chrono::Utc::now().timestamp();
    let memory = Memory {
        id: None,
        memory_type: Some(r#type),
        scope: Some(scope),
        key,
        content: Some(content),
        embedding: Some(emb_bytes),
        importance: Some(0.5),
        pinned: Some(0),
        source_message_id: None,
        created_at: now,
        last_used_at: None,
        use_count: Some(0),
    };

    let id = save(&pool, &memory).await.map_err(|e| e.to_string())?;
    Ok(id)
}

/// Semantic search across memories.
#[tauri::command]
pub async fn search_memories_semantic(
    query: String,
    top_k: Option<usize>,
    pool: tauri::State<'_, SqlitePool>,
) -> Result<Vec<RetrievedMemory>, String> {
    let api_key = llm::secrets::load_openai_api_key()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no LLM provider configured".to_string())?;
    let llm_provider = OpenAi::with_api_key(api_key);

    let k = top_k.unwrap_or(20);
    let results = retrieve(&pool, &llm_provider, &query, &["global".to_string()], k)
        .await
        .map_err(|e| e.to_string())?;

    Ok(results)
}

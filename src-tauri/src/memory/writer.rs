use sqlx::SqlitePool;
use tracing::{debug, instrument, warn};

use crate::error::AppResult;
use crate::llm::provider::LlmProvider;
use crate::storage::models::Memory;
use crate::storage::queries;

use super::retriever::retrieve;
use super::store::{encode_embedding, save};

// ---------------------------------------------------------------------------
// MemoryCandidate
//
// Also defined here; agents::reflection re-exports this type.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct MemoryCandidate {
    pub r#type: String,
    pub scope: String,
    pub key: Option<String>,
    pub content: String,
    pub importance: f32,
}

// ---------------------------------------------------------------------------
// Ingest
// ---------------------------------------------------------------------------

/// Embed and persist a batch of memory candidates, skipping near-duplicates.
///
/// Near-duplicate check: if an existing memory in the same (scope, type, key)
/// group has cosine similarity > 0.9, skip the new candidate and instead bump
/// the existing row's importance if the new candidate has higher importance.
#[instrument(skip(pool, llm, candidates), fields(n = candidates.len()))]
pub async fn ingest_candidates(
    pool: &SqlitePool,
    llm: &dyn LlmProvider,
    candidates: &[MemoryCandidate],
    source_msg_id: Option<i64>,
) -> AppResult<Vec<i64>> {
    if candidates.is_empty() {
        return Ok(vec![]);
    }

    // Batch-embed all candidate contents in one API call.
    let contents: Vec<String> = candidates.iter().map(|c| c.content.clone()).collect();
    let embeddings = llm.embed(None, contents).await?;

    let now = chrono::Utc::now().timestamp();
    let mut ids: Vec<i64> = Vec::with_capacity(candidates.len());

    for (candidate, embedding) in candidates.iter().zip(embeddings.iter()) {
        // Skip near-duplicates by checking existing memories with same scope.
        let scope_hints = [candidate.scope.clone()];
        let existing = retrieve(pool, llm, &candidate.content, &scope_hints, 5).await?;

        // Check for high-similarity duplicates in the same (type, key) group.
        let mut is_duplicate = false;
        for existing_mem in &existing {
            if existing_mem.score > 0.9 {
                // It's a near-duplicate; update importance if the new one is more important.
                debug!(
                    existing_id = existing_mem.id,
                    score = existing_mem.score,
                    "near-duplicate memory, skipping insertion"
                );
                // Bump importance if the new candidate is more important.
                let new_imp = candidate.importance as f64;
                queries::update_memory_importance(pool, existing_mem.id, new_imp.max(0.5))
                    .await
                    .unwrap_or_else(|e| warn!("failed to update importance: {e}"));
                is_duplicate = true;
                break;
            }
        }

        if is_duplicate {
            continue;
        }

        let emb_bytes = encode_embedding(embedding);

        let memory = Memory {
            id: None,
            memory_type: Some(candidate.r#type.clone()),
            scope: Some(candidate.scope.clone()),
            key: candidate.key.clone(),
            content: Some(candidate.content.clone()),
            embedding: Some(emb_bytes),
            importance: Some(candidate.importance as f64),
            pinned: Some(0),
            source_message_id: source_msg_id,
            created_at: now,
            last_used_at: None,
            use_count: Some(0),
        };

        let id = save(pool, &memory).await?;
        ids.push(id);
    }

    Ok(ids)
}

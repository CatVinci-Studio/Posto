use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{debug, instrument};

use crate::error::AppResult;
use crate::llm::provider::LlmProvider;
use crate::storage::models::Message;
use crate::storage::queries;

use super::store::decode_embedding;

// ---------------------------------------------------------------------------
// Output type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedMemory {
    pub id: i64,
    pub content: String,
    pub score: f32,
}

// ---------------------------------------------------------------------------
// Cosine similarity (in-memory)
//
// TODO: switch to sqlite-vec when the extension is bundled.
// ---------------------------------------------------------------------------

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Build and return a formatted memory block for an email message.
/// Always includes pinned memories; adds semantically similar ones up to `top_k`.
#[instrument(skip(pool, llm, msg), fields(msg_id = ?msg.id))]
pub async fn retrieve_for_message(
    pool: &SqlitePool,
    llm: &dyn LlmProvider,
    msg: &Message,
) -> AppResult<String> {
    let from_addr = msg.from_addr.as_deref().unwrap_or("");
    let subject = msg.subject.as_deref().unwrap_or("");
    let body_snippet: String = msg
        .body_text
        .as_deref()
        .unwrap_or("")
        .chars()
        .take(500)
        .collect();

    let query = format!("{subject} {from_addr} {body_snippet}");

    // Collect scope hints: global + any contact scope for the sender.
    let mut scope_hints = vec!["global".to_string()];
    if !from_addr.is_empty() {
        scope_hints.push(format!("contact:{from_addr}"));
    }

    let memories = retrieve(pool, llm, &query, &scope_hints, 20).await?;

    if memories.is_empty() {
        return Ok(String::new());
    }

    let formatted: String = memories
        .iter()
        .map(|m| format!("- {}", m.content))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(formatted)
}

/// Semantic retrieval: embed `query`, load candidate memories from DB,
/// rank by cosine similarity + always include pinned ones at the top.
pub async fn retrieve(
    pool: &SqlitePool,
    llm: &dyn LlmProvider,
    query: &str,
    scope_hints: &[String],
    top_k: usize,
) -> AppResult<Vec<RetrievedMemory>> {
    // Add "global" scope if not already present.
    let mut scopes = scope_hints.to_vec();
    if !scopes.contains(&"global".to_string()) {
        scopes.push("global".to_string());
    }

    // Load up to 500 candidate memories from DB.
    let candidates = queries::load_memories_by_scopes(pool, &scopes, 500).await?;
    if candidates.is_empty() {
        return Ok(vec![]);
    }

    // Embed the query.
    let query_vecs = llm.embed(None, vec![query.to_string()]).await?;
    let query_vec = query_vecs.into_iter().next().unwrap_or_default();

    debug!(
        candidates = candidates.len(),
        query_dim = query_vec.len(),
        "scoring memories"
    );

    // Score each memory; pinned ones get a bonus of +2.0 to always rank first.
    let mut scored: Vec<(RetrievedMemory, f32)> = candidates
        .into_iter()
        .filter_map(|m| {
            let id = m.id?;
            let content = m.content.clone().unwrap_or_default();
            let pinned = m.pinned.unwrap_or(0) != 0;

            let sim = if let Some(ref emb_bytes) = m.embedding {
                if emb_bytes.is_empty() {
                    0.0f32
                } else {
                    let emb = decode_embedding(emb_bytes);
                    cosine(&query_vec, &emb)
                }
            } else {
                0.0f32
            };

            let score = if pinned { sim + 2.0 } else { sim };
            Some((RetrievedMemory { id, content, score: sim }, score))
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_k);

    let results: Vec<RetrievedMemory> = scored.into_iter().map(|(r, _)| r).collect();
    Ok(results)
}


use sqlx::SqlitePool;

use crate::error::AppResult;
use crate::storage::models::Memory;
use crate::storage::queries;

// ---------------------------------------------------------------------------
// Embedding codec
// ---------------------------------------------------------------------------

/// Encode a `Vec<f32>` to little-endian bytes for SQLite BLOB storage.
pub fn encode_embedding(v: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(v.len() * 4);
    for f in v {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}

/// Decode a BLOB back to `Vec<f32>`.  Silently truncates to the nearest
/// complete float if the byte slice has an odd length.
pub fn decode_embedding(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| {
            let arr: [u8; 4] = chunk.try_into().unwrap();
            f32::from_le_bytes(arr)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// CRUD operations
// ---------------------------------------------------------------------------

/// Persist a Memory row.  If `memory.embedding` is already set as raw bytes,
/// it is stored as-is.  Returns the new row id.
pub async fn save(pool: &SqlitePool, memory: &Memory) -> AppResult<i64> {
    queries::insert_memory(pool, memory).await
}

/// Set or clear the `pinned` flag on a memory.
pub async fn pin(pool: &SqlitePool, id: i64, pinned: bool) -> AppResult<()> {
    queries::update_memory_pinned(pool, id, pinned).await
}

/// Hard-delete a memory by id.
pub async fn delete(pool: &SqlitePool, id: i64) -> AppResult<()> {
    queries::delete_memory_by_id(pool, id).await
}

/// Increment use_count and update last_used_at for a memory.
pub async fn touch(pool: &SqlitePool, id: i64) -> AppResult<()> {
    queries::touch_memory(pool, id).await
}

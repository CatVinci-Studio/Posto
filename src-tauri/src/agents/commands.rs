use sqlx::SqlitePool;
use tauri::AppHandle;

use crate::llm::{self, OpenAi};
use crate::storage::models::AgentRun;
use crate::storage::queries;

use super::pipeline::{run_pipeline, PipelineOutput};

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Trigger the full agent pipeline for a single message.
///
/// Returns an error string if no LLM API key is configured.
#[tauri::command]
pub async fn trigger_agent_pipeline(
    message_id: i64,
    pool: tauri::State<'_, SqlitePool>,
    app: AppHandle,
) -> Result<PipelineOutput, String> {
    let key = llm::secrets::load_openai_api_key()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no LLM provider configured".to_string())?;

    let llm_provider = OpenAi::with_api_key(key);
    run_pipeline(&pool, &llm_provider, message_id, &app)
        .await
        .map_err(|e| e.to_string())
}

/// List AgentRun rows, optionally filtered by message_id.
#[tauri::command]
pub async fn list_agent_runs(
    message_id: Option<i64>,
    limit: Option<i64>,
    pool: tauri::State<'_, SqlitePool>,
) -> Result<Vec<AgentRun>, String> {
    let limit = limit.unwrap_or(50);
    queries::list_agent_runs(&pool, message_id, limit)
        .await
        .map_err(|e| e.to_string())
}

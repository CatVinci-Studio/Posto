use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter};
use tracing::{info, instrument, warn};

use crate::error::AppResult;
use crate::llm::provider::LlmProvider;
use crate::storage::models::{AgentRun, Task};
use crate::storage::queries;

use super::action::ActionOutput;
use super::reflection::ReflectionOutput;
use super::summary::SummaryOutput;
use super::triage::TriageOutput;
use super::{action, reflection, summary, triage};
use crate::memory;
use crate::memory::writer::MemoryCandidate;

// ---------------------------------------------------------------------------
// Output type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineOutput {
    pub triage: TriageOutput,
    pub summary: SummaryOutput,
    pub action: ActionOutput,
    pub reflection: ReflectionOutput,
    pub total_tokens_in: u32,
    pub total_tokens_out: u32,
}

// ---------------------------------------------------------------------------
// Pipeline runner
// ---------------------------------------------------------------------------

/// Run the full agent pipeline for a single email message.
///
/// Steps:
///   1. Load message from DB.
///   2. Retrieve relevant memories.
///   3. Run Triage (required first — provides language).
///   4. Run Summary + Action in parallel.
///   5. Run Reflection over combined outputs.
///   6. Persist AgentRun rows.
///   7. Insert tasks from ActionOutput.
///   8. Ingest reflection candidates into memory store.
///   9. Update detected_lang if previously null.
///  10. Emit "email:processed" event.
#[instrument(skip(pool, llm, app), fields(msg_id))]
pub async fn run_pipeline(
    pool: &SqlitePool,
    llm: &dyn LlmProvider,
    msg_id: i64,
    app: &AppHandle,
) -> AppResult<PipelineOutput> {
    // 1. Load message.
    let msg = queries::get_message(pool, msg_id).await?;

    // 2. Retrieve memories.
    let memories_block = memory::retriever::retrieve_for_message(pool, llm, &msg).await?;

    // 3. Triage (sequential — others depend on its output for context).
    let triage_out = triage::run(llm, &msg, &memories_block).await?;
    info!(
        category = %triage_out.category,
        priority = triage_out.priority,
        "triage complete"
    );

    // 4. Summary + Action in parallel.
    let (summary_out, action_out) = tokio::join!(
        summary::run(llm, &msg, &memories_block),
        action::run(llm, &msg, &memories_block),
    );
    let summary_out = summary_out?;
    let action_out = action_out?;

    // 5. Reflection.
    let reflection_out =
        reflection::run(llm, &msg, &triage_out, &summary_out, &action_out).await?;

    // 6. Persist AgentRun rows.
    let now = chrono::Utc::now().timestamp();
    let model = llm.default_model().to_string();

    let triage_json = serde_json::to_string(&triage_out).unwrap_or_default();
    let summary_json = serde_json::to_string(&summary_out).unwrap_or_default();
    let action_json = serde_json::to_string(&action_out).unwrap_or_default();
    let reflection_json = serde_json::to_string(&reflection_out).unwrap_or_default();

    for (agent_type, output_json) in &[
        ("triage", &triage_json),
        ("summary", &summary_json),
        ("action", &action_json),
        ("reflection", &reflection_json),
    ] {
        let run = AgentRun {
            id: None,
            message_id: Some(msg_id),
            agent_type: Some(agent_type.to_string()),
            input_summary: msg.subject.clone(),
            output: Some(output_json.to_string()),
            tokens_in: None,
            tokens_out: None,
            model: Some(model.clone()),
            cost: None,
            created_at: now,
        };
        if let Err(e) = queries::insert_agent_run(pool, &run).await {
            warn!(agent_type, error = %e, "failed to persist AgentRun");
        }
    }

    // 7. Insert tasks extracted by action agent.
    for task_item in &action_out.tasks {
        // Parse due_at string to unix timestamp if provided.
        let due_at_ts: Option<i64> = task_item
            .due_at
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp());

        let task = Task {
            id: None,
            message_id: Some(msg_id),
            account_id: Some(msg.account_id),
            title: Some(task_item.title.clone()),
            due_at: due_at_ts,
            status: Some("open".to_string()),
            created_at: now,
        };
        if let Err(e) = queries::insert_task(pool, &task).await {
            warn!(title = %task_item.title, error = %e, "failed to insert task");
        }
    }

    // 8. Ingest reflection candidates.
    let candidates: Vec<MemoryCandidate> = reflection_out
        .candidates
        .iter()
        .map(|c| MemoryCandidate {
            r#type: c.r#type.clone(),
            scope: c.scope.clone(),
            key: c.key.clone(),
            content: c.content.clone(),
            importance: c.importance,
        })
        .collect();

    if let Err(e) =
        memory::writer::ingest_candidates(pool, llm, &candidates, Some(msg_id)).await
    {
        warn!(error = %e, "failed to ingest memory candidates");
    }

    // 9. Update detected_lang if currently null.
    if let Some(ref lang) = triage_out.language {
        if let Err(e) = queries::update_message_lang(pool, msg_id, lang).await {
            warn!(error = %e, "failed to update detected_lang");
        }
    }

    // 10. Emit event.
    let event_payload = serde_json::json!({
        "message_id": msg_id,
        "summary": summary_out.summary_en,
    });
    if let Err(e) = app.emit("email:processed", event_payload) {
        warn!(error = %e, "failed to emit email:processed event");
    }

    Ok(PipelineOutput {
        triage: triage_out,
        summary: summary_out,
        action: action_out,
        reflection: reflection_out,
        total_tokens_in: 0,  // TODO: accumulate from CompletionResponse.usage when tracked
        total_tokens_out: 0,
    })
}

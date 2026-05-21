use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::error::{AppError, AppResult};
use crate::llm::provider::{CompletionRequest, LlmProvider, Message, ResponseFormat, Role};
use crate::storage::models::Message as EmailMessage;

use super::prompts::TRIAGE_SYSTEM;

// ---------------------------------------------------------------------------
// Output type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageOutput {
    pub category: String,
    pub priority: u32,
    pub labels: Vec<String>,
    pub is_actionable: bool,
    pub language: Option<String>,
}

// Internal deserialization target (priority might come as float from the model)
#[derive(Debug, Deserialize)]
struct TriageRaw {
    category: String,
    priority: serde_json::Value,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    is_actionable: bool,
    language: Option<String>,
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

/// Run the triage agent for a single email message.
///
/// `memories` is a pre-formatted block of relevant long-term memories
/// (can be empty string if none available).
#[instrument(skip(llm, msg, memories), fields(msg_id = ?msg.id))]
pub async fn run(
    llm: &dyn LlmProvider,
    msg: &EmailMessage,
    memories: &str,
) -> AppResult<TriageOutput> {
    let subject = msg.subject.as_deref().unwrap_or("(no subject)");
    let from_name = msg.from_name.as_deref().unwrap_or("");
    let from_addr = msg.from_addr.as_deref().unwrap_or("");

    // Prefer snippet; fall back to body_text truncated to 4000 chars.
    let body_preview: String = msg
        .snippet
        .as_deref()
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            msg.body_text
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(4000)
                .collect()
        });

    let user_content = build_user_message(subject, from_name, from_addr, memories, &body_preview);

    let req = CompletionRequest {
        model: llm.default_model().to_string(),
        messages: vec![
            Message {
                role: Role::System,
                content: Some(TRIAGE_SYSTEM.to_string()),
                tool_calls: vec![],
                tool_call_id: None,
                name: None,
            },
            Message {
                role: Role::User,
                content: Some(user_content),
                tool_calls: vec![],
                tool_call_id: None,
                name: None,
            },
        ],
        tools: vec![],
        temperature: Some(0.1),
        max_tokens: Some(512),
        response_format: Some(ResponseFormat::JsonObject),
    };

    let resp = llm.complete(req).await?;
    let raw_json = resp
        .content
        .ok_or_else(|| AppError::Provider("triage: empty response content".into()))?;

    let raw: TriageRaw = serde_json::from_str(&raw_json)
        .map_err(|e| AppError::Json(e))?;

    let priority = match &raw.priority {
        serde_json::Value::Number(n) => n.as_u64().unwrap_or(50) as u32,
        serde_json::Value::String(s) => s.parse::<u32>().unwrap_or(50),
        _ => 50,
    };

    Ok(TriageOutput {
        category: raw.category,
        priority: priority.min(100),
        labels: raw.labels,
        is_actionable: raw.is_actionable,
        language: raw.language,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_user_message(
    subject: &str,
    from_name: &str,
    from_addr: &str,
    memories: &str,
    body_preview: &str,
) -> String {
    let memories_block = if memories.is_empty() {
        "(none)".to_string()
    } else {
        memories.to_string()
    };

    format!(
        "Subject: {subject}\n\
         From: {from_name} <{from_addr}>\n\n\
         Memories about this contact and your preferences:\n\
         {memories_block}\n\n\
         Email body:\n\
         {body_preview}"
    )
}

use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::error::{AppError, AppResult};
use crate::llm::provider::{CompletionRequest, LlmProvider, Message, ResponseFormat, Role};
use crate::storage::models::Message as EmailMessage;

use super::prompts::ACTION_SYSTEM;

// ---------------------------------------------------------------------------
// Output types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskItem {
    pub title: String,
    pub due_at: Option<String>,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionOutput {
    #[serde(default)]
    pub tasks: Vec<TaskItem>,
    #[serde(default)]
    pub needs_response: bool,
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

/// Run the action-extraction agent for a single email message.
#[instrument(skip(llm, msg, memories), fields(msg_id = ?msg.id))]
pub async fn run(
    llm: &dyn LlmProvider,
    msg: &EmailMessage,
    memories: &str,
) -> AppResult<ActionOutput> {
    let subject = msg.subject.as_deref().unwrap_or("(no subject)");
    let from_name = msg.from_name.as_deref().unwrap_or("");
    let from_addr = msg.from_addr.as_deref().unwrap_or("");

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

    let memories_block = if memories.is_empty() {
        "(none)".to_string()
    } else {
        memories.to_string()
    };

    let user_content = format!(
        "Subject: {subject}\n\
         From: {from_name} <{from_addr}>\n\n\
         Memories about this contact and your preferences:\n\
         {memories_block}\n\n\
         Email body:\n\
         {body_preview}"
    );

    let req = CompletionRequest {
        model: llm.default_model().to_string(),
        messages: vec![
            Message {
                role: Role::System,
                content: Some(ACTION_SYSTEM.to_string()),
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
        max_tokens: Some(1024),
        response_format: Some(ResponseFormat::JsonObject),
    };

    let resp = llm.complete(req).await?;
    let raw_json = resp
        .content
        .ok_or_else(|| AppError::Provider("action: empty response content".into()))?;

    let output: ActionOutput =
        serde_json::from_str(&raw_json).map_err(|e| AppError::Json(e))?;

    Ok(output)
}
